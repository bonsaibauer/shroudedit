use std::collections::HashSet;
use std::io::{Cursor, Read};
use std::sync::LazyLock;

use bytes::Bytes;
use serde::Deserialize;
use serde_json::Value;
use zip::ZipArchive;

const SCHEMA: &str = include_str!("schemas/sfpack/1/manifest.schema.json");
const MAX_ARCHIVE_BYTES: usize = 2 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

static VALIDATOR: LazyLock<jsonschema::Validator> = LazyLock::new(|| {
	jsonschema::validator_for(
		&serde_json::from_str::<Value>(SCHEMA)
			.expect("bundled sfpack schema is JSON"),
	)
	.expect("bundled sfpack schema is valid")
});

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMod {
	pub project_id: String,
	pub version_id: String,
	pub sha512: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pack {
	pub version_id: String,
	pub game_versions: Vec<String>,
	pub mods: Vec<PackMod>,
}

pub fn validate_package(
	data: Bytes,
	expected_version: Option<&str>,
) -> Result<Pack, String> {
	if data.len() > MAX_ARCHIVE_BYTES {
		return Err("The .sfpack archive exceeds 2 MiB".into());
	}
	let mut archive = ZipArchive::new(Cursor::new(data))
		.map_err(|error| format!("Invalid .sfpack ZIP: {error}"))?;
	if archive.len() != 1 {
		return Err(
			"A .sfpack archive must contain only shroudforge.index.json".into(),
		);
	}
	let entry = archive
		.by_name("shroudforge.index.json")
		.map_err(|_| "The .sfpack archive has no shroudforge.index.json")?;
	if entry.is_dir()
		|| entry
			.unix_mode()
			.is_some_and(|mode| mode & 0o170000 == 0o120000)
		|| !matches!(
			entry.compression(),
			zip::CompressionMethod::Stored | zip::CompressionMethod::Deflated
		) || entry.size() > MAX_MANIFEST_BYTES
	{
		return Err("Invalid .sfpack manifest entry".into());
	}
	let mut contents = Vec::new();
	entry
		.take(MAX_MANIFEST_BYTES + 1)
		.read_to_end(&mut contents)
		.map_err(|error| format!("Could not read .sfpack manifest: {error}"))?;
	if contents.len() as u64 > MAX_MANIFEST_BYTES {
		return Err("The .sfpack manifest exceeds 1 MiB".into());
	}
	let value: Value = serde_json::from_slice(&contents)
		.map_err(|error| format!("Invalid .sfpack JSON: {error}"))?;
	VALIDATOR.validate(&value).map_err(|error| {
		format!("Invalid .sfpack schema at {}: {error}", error.instance_path)
	})?;
	let pack: Pack =
		serde_json::from_value(value).map_err(|error| error.to_string())?;
	if expected_version.is_some_and(|version| pack.version_id != version) {
		return Err(format!(
			".sfpack versionId '{}' must match the published version",
			pack.version_id
		));
	}
	let mut projects = HashSet::new();
	for item in &pack.mods {
		if !projects.insert(&item.project_id) {
			return Err(format!(
				"Duplicate .sfpack project ID '{}'",
				item.project_id
			));
		}
	}
	Ok(pack)
}
