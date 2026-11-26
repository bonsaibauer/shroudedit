use std::io::{Cursor, Read};
use std::sync::LazyLock;

use bytes::Bytes;
use serde::Deserialize;
use serde_json::Value;
use zip::ZipArchive;

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MANIFEST_SCHEMA: &str =
	include_str!("schemas/shroudforge-1.8.2/manifest.schema.json");
const EXTENSION_SCHEMA: &str =
	include_str!("schemas/shroudforge-1.8.2/extended.mod.schema.json");

static MANIFEST_VALIDATOR: LazyLock<jsonschema::Validator> =
	LazyLock::new(|| {
		jsonschema::validator_for(
			&serde_json::from_str::<Value>(MANIFEST_SCHEMA)
				.expect("bundled manifest schema is JSON"),
		)
		.expect("bundled manifest schema is valid")
	});

static EXTENSION_VALIDATOR: LazyLock<jsonschema::Validator> =
	LazyLock::new(|| {
		jsonschema::validator_for(
			&serde_json::from_str::<Value>(EXTENSION_SCHEMA)
				.expect("bundled extension schema is JSON"),
		)
		.expect("bundled extension schema is valid")
	});

#[derive(Deserialize)]
pub struct PackageDependency {
	pub id: String,
	pub version: String,
	#[serde(default)]
	pub optional: bool,
}

pub struct PackageMetadata {
	pub dependencies: Vec<PackageDependency>,
}

fn read_json(
	archive: &mut ZipArchive<Cursor<Bytes>>,
	path: &str,
) -> Result<Value, String> {
	let entry = archive
		.by_name(path)
		.map_err(|error| format!("{path}: {error}"))?;
	let mut contents = Vec::new();
	entry
		.take(MAX_MANIFEST_BYTES + 1)
		.read_to_end(&mut contents)
		.map_err(|error| format!("{path}: {error}"))?;
	if contents.len() as u64 > MAX_MANIFEST_BYTES {
		return Err(format!("{path} exceeds the 1 MiB manifest limit"));
	}
	serde_json::from_slice(&contents)
		.map_err(|error| format!("{path}: {error}"))
}

pub fn validate_package(
	data: Bytes,
	expected_slug: Option<&str>,
	expected_version: &str,
) -> Result<Option<PackageMetadata>, String> {
	let mut archive = ZipArchive::new(Cursor::new(data))
		.map_err(|error| format!("Invalid ShroudForge ZIP: {error}"))?;
	let manifests = archive
		.file_names()
		.filter(|path| *path == "mod.json" || path.ends_with("/mod.json"))
		.map(str::to_owned)
		.collect::<Vec<_>>();
	let manifest_path = match manifests.as_slice() {
		[path] => path,
		[] => {
			let extensions = archive
				.file_names()
				.filter(|path| {
					*path == "extended.mod.json"
						|| path.ends_with("/extended.mod.json")
				})
				.map(str::to_owned)
				.collect::<Vec<_>>();
			if extensions.len() > 1 {
				return Err(
					"The ZIP contains more than one extended.mod.json".into()
				);
			}
			if let Some(path) = extensions.first() {
				let extension = read_json(&mut archive, path)?;
				EXTENSION_VALIDATOR.validate(&extension).map_err(|error| {
					format!(
						"{path} does not match schema 1.8.2 at {}: {error}",
						error.instance_path
					)
				})?;
			}
			return Ok(None);
		}
		_ => return Err("The ZIP contains more than one mod.json".into()),
	};
	let manifest = read_json(&mut archive, manifest_path)?;
	MANIFEST_VALIDATOR.validate(&manifest).map_err(|error| {
		format!(
			"{manifest_path} does not match schema 1.8.2 at {}: {error}",
			error.instance_path
		)
	})?;
	let slug =
		expected_slug.ok_or("The project has no slug for its mod.json ID")?;
	let id = manifest["id"]
		.as_str()
		.ok_or("mod.json.id must be a string")?;
	if id != slug.to_ascii_lowercase() {
		return Err(format!(
			"mod.json.id '{id}' must match project slug '{slug}'"
		));
	}
	let version = manifest["version"]
		.as_str()
		.ok_or("mod.json.version must be a string")?;
	if version != expected_version {
		return Err(format!(
			"mod.json.version '{version}' must match published version '{expected_version}'"
		));
	}
	let dependencies = serde_json::from_value::<Vec<PackageDependency>>(
		manifest
			.get("dependencies")
			.cloned()
			.unwrap_or_else(|| Value::Array(Vec::new())),
	)
	.map_err(|error| format!("mod.json.dependencies: {error}"))?;
	for dependency in &dependencies {
		if dependency.id != "shroudforge-api" {
			semver::VersionReq::parse(&dependency.version).map_err(
				|error| {
					format!(
						"Invalid version requirement for '{}': {error}",
						dependency.id
					)
				},
			)?;
		}
	}
	let prefix = manifest_path
		.strip_suffix("mod.json")
		.expect("manifest suffix was checked");
	let extension_path = format!("{prefix}extended.mod.json");
	if archive.file_names().any(|path| path == extension_path) {
		let extension = read_json(&mut archive, &extension_path)?;
		EXTENSION_VALIDATOR.validate(&extension).map_err(|error| {
			format!(
				"{extension_path} does not match schema 1.8.2 at {}: {error}",
				error.instance_path
			)
		})?;
	}
	Ok(Some(PackageMetadata { dependencies }))
}
