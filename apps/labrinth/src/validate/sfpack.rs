use std::collections::HashSet;
use std::io::{Cursor, Read};
use std::sync::LazyLock;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zip::ZipArchive;

const SCHEMA: &str =
	include_str!("schemas/shroudforge-1.8.2/modpack.schema.json");
const MOD_SCHEMA: &str =
	include_str!("schemas/shroudforge-1.8.2/manifest.schema.json");
const EXTENSION_SCHEMA: &str =
	include_str!("schemas/shroudforge-1.8.2/extended.mod.schema.json");
const MAX_ARCHIVE_BYTES: usize = 95 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_EXPANDED_BYTES: u64 = 512 * 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;

fn make_validator(schema: &str) -> jsonschema::Validator {
	jsonschema::validator_for(
		&serde_json::from_str::<Value>(schema).expect("bundled schema is JSON"),
	)
	.expect("bundled schema is valid")
}

static VALIDATOR: LazyLock<jsonschema::Validator> =
	LazyLock::new(|| make_validator(SCHEMA));
static MOD_VALIDATOR: LazyLock<jsonschema::Validator> =
	LazyLock::new(|| make_validator(MOD_SCHEMA));
static EXTENSION_VALIDATOR: LazyLock<jsonschema::Validator> =
	LazyLock::new(|| make_validator(EXTENSION_SCHEMA));

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pack {
	pub id: String,
	pub name: String,
	pub version: String,
	pub game_versions: Vec<String>,
	pub mods: Vec<PackMod>,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum PackMod {
	Bundled {
		path: String,
		#[serde(default)]
		optional: bool,
	},
	External {
		id: String,
		version: String,
		url: String,
		sha512: String,
		#[serde(default)]
		optional: bool,
	},
	Catalog {
		id: String,
		version: String,
		#[serde(default)]
		optional: bool,
	},
}

impl PackMod {
	pub fn id(&self) -> &str {
		match self {
			Self::Bundled { path, .. } => {
				path.rsplit('/').next().unwrap_or(path)
			}
			Self::External { id, .. } | Self::Catalog { id, .. } => id,
		}
	}

	pub fn optional(&self) -> bool {
		match self {
			Self::Bundled { optional, .. }
			| Self::External { optional, .. }
			| Self::Catalog { optional, .. } => *optional,
		}
	}
}

fn read_json(
	archive: &mut ZipArchive<Cursor<Bytes>>,
	path: &str,
) -> Result<Value, String> {
	let entry = archive
		.by_name(path)
		.map_err(|_| format!("missing {path}"))?;
	if entry.size() > MAX_MANIFEST_BYTES {
		return Err(format!("{path} exceeds 1 MiB"));
	}
	let mut contents = Vec::new();
	entry
		.take(MAX_MANIFEST_BYTES + 1)
		.read_to_end(&mut contents)
		.map_err(|error| format!("could not read {path}: {error}"))?;
	if contents.len() as u64 > MAX_MANIFEST_BYTES {
		return Err(format!("{path} exceeds 1 MiB"));
	}
	serde_json::from_slice(&contents)
		.map_err(|error| format!("invalid {path} JSON: {error}"))
}

pub fn validate_package(
	data: Bytes,
	expected_slug: Option<&str>,
	expected_version: Option<&str>,
) -> Result<Pack, String> {
	if data.len() > MAX_ARCHIVE_BYTES {
		return Err("the .sfpack archive exceeds 95 MiB".into());
	}
	let mut archive = ZipArchive::new(Cursor::new(data))
		.map_err(|error| format!("invalid .sfpack ZIP: {error}"))?;
	if archive.len() > MAX_ENTRIES {
		return Err("the .sfpack archive contains too many entries".into());
	}
	let manifest = read_json(&mut archive, "modpack.json")?;
	VALIDATOR.validate(&manifest).map_err(|error| {
		format!("invalid modpack.json at {}: {error}", error.instance_path)
	})?;
	let pack: Pack = serde_json::from_value(manifest)
		.map_err(|error| format!("invalid modpack.json: {error}"))?;
	if expected_slug.is_some_and(|slug| pack.id != slug.to_ascii_lowercase()) {
		return Err("modpack.json id must match the project slug".into());
	}
	if expected_version.is_some_and(|version| pack.version != version) {
		return Err(
			"modpack.json version must match the published version".into()
		);
	}
	let mut ids = HashSet::new();
	let bundled = pack
		.mods
		.iter()
		.filter_map(|item| match item {
			PackMod::Bundled { path, .. } => Some(path.as_str()),
			_ => None,
		})
		.collect::<Vec<_>>();
	for item in &pack.mods {
		if !ids.insert(item.id().to_ascii_lowercase()) {
			return Err(format!("duplicate mod {} in modpack.json", item.id()));
		}
		if let PackMod::External { url, .. } = item {
			let parsed = url::Url::parse(url).map_err(|error| {
				format!("invalid external mod URL: {error}")
			})?;
			if parsed.scheme() != "https"
				|| parsed.host_str().is_none()
				|| !parsed.username().is_empty()
				|| parsed.password().is_some()
			{
				return Err(format!(
					"external mod {} needs an HTTPS URL without credentials",
					item.id()
				));
			}
		}
	}

	let mut names = HashSet::new();
	let mut payloads = HashSet::new();
	let mut expanded = 0_u64;
	for index in 0..archive.len() {
		let mut entry = archive
			.by_index(index)
			.map_err(|error| format!("invalid ZIP entry: {error}"))?;
		let name = entry.name().to_owned();
		let path = name.trim_end_matches('/');
		if name.starts_with('/')
			|| name.len() > 4096
			|| name.contains('\\')
			|| name.chars().any(|character| {
				character.is_control()
					|| matches!(
						character,
						':' | '<' | '>' | '"' | '?' | '*' | '|'
					)
			}) || path
			.split('/')
			.any(|part| part.is_empty() || matches!(part, "." | ".."))
			|| path
				.split('/')
				.any(|part| part.ends_with(' ') || part.ends_with('.'))
			|| path_util::SafeRelativeUtf8UnixPathBuf::try_from(path.to_owned())
				.is_err()
			|| !names.insert(path.to_ascii_lowercase())
			|| entry
				.unix_mode()
				.is_some_and(|mode| mode & 0o170000 == 0o120000)
			|| !matches!(
				entry.compression(),
				zip::CompressionMethod::Stored
					| zip::CompressionMethod::Deflated
			) {
			return Err(format!("invalid .sfpack entry {name}"));
		}
		let owner = bundled.iter().find(|root| {
			path == *root
				|| path
					.strip_prefix(*root)
					.is_some_and(|rest| rest.starts_with('/'))
		});
		if name != "modpack.json"
			&& owner.is_none()
			&& !(entry.is_dir() && path == "mods")
		{
			return Err(format!("undeclared .sfpack entry {name}"));
		}
		if owner.is_some_and(|root| path == *root) && !entry.is_dir() {
			return Err(format!("bundled mod root {name} must be a directory"));
		}
		if name == "modpack.json" && entry.is_dir() {
			return Err("modpack.json must be a file".into());
		}
		if entry.is_dir() {
			continue;
		}
		if let Some(root) = owner {
			payloads.insert(*root);
		}
		expanded = expanded
			.checked_add(entry.size())
			.ok_or("expanded .sfpack size overflow")?;
		if expanded > MAX_EXPANDED_BYTES {
			return Err("expanded .sfpack contents exceed 512 MiB".into());
		}
		let read = std::io::copy(
			&mut entry.take(MAX_EXPANDED_BYTES + 1),
			&mut std::io::sink(),
		)
		.map_err(|error| format!("could not read {name}: {error}"))?;
		if read > MAX_EXPANDED_BYTES {
			return Err(format!(".sfpack entry {name} exceeds 512 MiB"));
		}
	}

	for root in bundled {
		if !payloads.contains(root) {
			return Err(format!(
				"bundled mod folder {root} is empty or missing"
			));
		}
		let mod_path = format!("{root}/mod.json");
		if archive.file_names().any(|name| name == mod_path) {
			let value = read_json(&mut archive, &mod_path)?;
			MOD_VALIDATOR.validate(&value).map_err(|error| {
				format!(
					"invalid {mod_path} at {}: {error}",
					error.instance_path
				)
			})?;
			if value["id"].as_str() != root.rsplit('/').next() {
				return Err(format!("{mod_path} id must match its folder"));
			}
		}
		let extension_path = format!("{root}/extended.mod.json");
		if archive.file_names().any(|name| name == extension_path) {
			let value = read_json(&mut archive, &extension_path)?;
			EXTENSION_VALIDATOR.validate(&value).map_err(|error| {
				format!(
					"invalid {extension_path} at {}: {error}",
					error.instance_path
				)
			})?;
		}
	}
	Ok(pack)
}
