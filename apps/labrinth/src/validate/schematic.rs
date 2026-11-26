use crate::database::models::loader_fields::{
	LoaderFieldEnumValue, VersionField, VersionFieldValue,
};
use crate::models::projects::Loader;
use crate::validate::{ValidationError, ValidationResult};
use bytes::Bytes;
use std::borrow::Cow;

const SUPPORTED_LOADERS: &[&str] = &["shroudtopia", "shroudforge", "eml"];

fn invalid(message: impl Into<String>) -> ValidationError {
	ValidationError::InvalidInput(Cow::Owned(message.into()))
}

fn integer_field(fields: &[VersionField], name: &str) -> Option<i32> {
	fields.iter().find_map(|field| {
		(field.field_name == name)
			.then_some(&field.value)
			.and_then(|value| match value {
				VersionFieldValue::Integer(value) => Some(*value),
				_ => None,
			})
	})
}

fn text_field<'a>(fields: &'a [VersionField], name: &str) -> Option<&'a str> {
	fields.iter().find_map(|field| {
		(field.field_name == name)
			.then_some(&field.value)
			.and_then(|value| match value {
				VersionFieldValue::Text(value) => Some(value.as_str()),
				_ => None,
			})
	})
}

fn game_versions(fields: &[VersionField]) -> Option<&[LoaderFieldEnumValue]> {
	fields.iter().find_map(|field| {
		(field.field_name == "game_versions")
			.then_some(&field.value)
			.and_then(|value| match value {
				VersionFieldValue::ArrayEnum(_, values) => {
					Some(values.as_slice())
				}
				_ => None,
			})
	})
}

pub async fn validate_file(
	data: Bytes,
	loaders: Vec<Loader>,
	fields: Vec<VersionField>,
) -> Result<ValidationResult, ValidationError> {
	if loaders.is_empty()
		|| loaders
			.iter()
			.any(|loader| !SUPPORTED_LOADERS.contains(&loader.0.as_str()))
	{
		return Err(invalid(
			"SFBP requires the shroudtopia, shroudforge, or eml loader",
		));
	}
	if game_versions(&fields).is_none_or(|versions| versions.len() != 1) {
		return Err(invalid(
			"SFBP blueprints must target exactly one game version",
		));
	}
	if integer_field(&fields, "schematic_format_version") != Some(8) {
		return Err(invalid("new SFBP uploads must use JSON format 8"));
	}
	if text_field(&fields, "world_editor_version")
		.is_none_or(|value| value.trim().is_empty())
	{
		return Err(invalid("world editor version cannot be empty"));
	}
	let metadata =
		actix_web::web::block(move || super::schematic_v8::validate(&data))
			.await?
			.map_err(invalid)?;
	let dimensions = metadata
		.voxel_dimensions
		.map(|values| values.map(f64::from))
		.unwrap_or_else(|| metadata.extent.map(f64::ceil));
	for (index, name) in
		["schematic_width", "schematic_height", "schematic_depth"]
			.iter()
			.enumerate()
	{
		let actual = dimensions[index];
		if !(1.0..=65536.0).contains(&actual)
			|| integer_field(&fields, name).map(f64::from) != Some(actual)
		{
			return Err(invalid(format!(
				"SFBP dimensions do not match the published `{name}`"
			)));
		}
	}
	Ok(ValidationResult::Pass)
}
