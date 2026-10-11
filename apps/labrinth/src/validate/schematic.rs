use crate::database::models::loader_fields::{
	LoaderFieldEnumValue, VersionField, VersionFieldValue,
};
use crate::models::projects::Loader;
use crate::validate::{ValidationError, ValidationResult};
use bytes::Bytes;
use std::borrow::Cow;

const FORMAT_HEADER: &str = "SHROUDFORGE_WORLD_BLUEPRINT_V7";
const MAXIMUM_VOXEL_CELLS: u64 = 65_536;
const MAXIMUM_PROPS: usize = 1_000_000;
const MAXIMUM_DIMENSION: u32 = 65_536;
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
				VersionFieldValue::ArrayEnum(_, values) => Some(values.as_slice()),
				_ => None,
			})
	})
}

fn parse_finite_number(value: &str) -> Option<f64> {
	let number = value.parse::<f64>().ok()?;
	number.is_finite().then_some(number)
}

fn parse_triplet(value: &str) -> Option<[f64; 3]> {
	let mut values = value.split(',');
	let triplet = [
		parse_finite_number(values.next()?)?,
		parse_finite_number(values.next()?)?,
		parse_finite_number(values.next()?)?,
	];
	values.next().is_none().then_some(triplet)
}

fn parse_dimensions(value: &str) -> Option<[u32; 3]> {
	let mut values = value.split(',');
	let dimensions = [
		values.next()?.parse::<u32>().ok()?,
		values.next()?.parse::<u32>().ok()?,
		values.next()?.parse::<u32>().ok()?,
	];
	values.next().is_none().then_some(dimensions)
}

fn validate_voxel_channel(
	dimensions: [u32; 3],
	offset: [f64; 3],
	cell_size: [f64; 3],
	cells: &str,
	coverage: &str,
) -> Option<[u32; 3]> {
	let cell_count = u64::from(dimensions[0])
		.checked_mul(u64::from(dimensions[1]))?
		.checked_mul(u64::from(dimensions[2]))?;
	if dimensions.contains(&0)
		|| dimensions.iter().any(|dimension| *dimension > MAXIMUM_DIMENSION)
		|| cell_count > MAXIMUM_VOXEL_CELLS
		|| offset.iter().any(|value| !value.is_finite())
		|| cell_size.iter().any(|value| !value.is_finite() || *value <= 0.0)
	{
		return None;
	}

	let mut parsed_cells = cells.split(',');
	let mut parsed_coverage = coverage.split(',');
	for _ in 0..cell_count {
		let cell = parse_finite_number(parsed_cells.next()?)?;
		let coverage = parse_finite_number(parsed_coverage.next()?)?;
		if cell.fract() != 0.0
			|| !(0.0..=f64::from(u16::MAX)).contains(&cell)
			|| coverage.fract() != 0.0
			|| !(0.0..=2.0).contains(&coverage)
			|| (coverage == 2.0 && cell == 0.0)
		{
			return None;
		}
	}
	if parsed_cells.next().is_some() || parsed_coverage.next().is_some() {
		return None;
	}

	Some(dimensions)
}

fn validate_props<'a>(
	lines: impl Iterator<Item = &'a str>,
	prop_count: usize,
) -> bool {
	let mut lines = lines;
	for _ in 0..prop_count {
		let Some(line) = lines.next() else {
			return false;
		};
		let mut fields = line.split(',');
		let Some(item_id) = fields.next().and_then(parse_finite_number) else {
			return false;
		};
		if item_id < 1.0 || item_id.fract() != 0.0 {
			return false;
		}

		let mut transform = [0.0_f64; 10];
		for value in &mut transform {
			let Some(parsed) = fields.next().and_then(parse_finite_number) else {
				return false;
			};
			*value = parsed;
		}
		let rotation_norm = transform[3..7]
			.iter()
			.map(|value| value * value)
			.sum::<f64>();
		if !rotation_norm.is_finite() || rotation_norm < 1e-12 {
			return false;
		}

		let (Some(high), Some(low)) = (fields.next(), fields.next()) else {
			return false;
		};
		if high.is_empty()
			|| low.is_empty()
			|| high.len() > 16
			|| low.len() > 16
			|| !high.bytes().all(|byte| byte.is_ascii_hexdigit())
			|| !low.bytes().all(|byte| byte.is_ascii_hexdigit())
			|| (high.bytes().all(|byte| byte == b'0')
				&& low.bytes().all(|byte| byte == b'0'))
		{
			return false;
		}
		if fields.next().is_some() {
			return false;
		}
	}
	lines.next().is_none()
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
			"SFBP blueprints require the shroudtopia, shroudforge, or eml loader",
		));
	}

	if game_versions(&fields).is_none_or(|versions| versions.len() != 1) {
		return Err(invalid("SFBP blueprints must target exactly one game version"));
	}

	if integer_field(&fields, "schematic_format_version") != Some(7) {
		return Err(invalid("SFBP blueprints must use format version 7"));
	}

	if text_field(&fields, "world_editor_version")
		.is_none_or(|value| value.trim().is_empty())
	{
		return Err(invalid("World Editor version cannot be empty"));
	}

	let metadata_dimensions = [
		integer_field(&fields, "schematic_width"),
		integer_field(&fields, "schematic_height"),
		integer_field(&fields, "schematic_depth"),
	];
	if metadata_dimensions.iter().any(Option::is_none) {
		return Err(invalid("SFBP blueprint dimensions are required"));
	}
	let metadata_dimensions = metadata_dimensions.map(Option::unwrap);
	if metadata_dimensions
		.iter()
		.any(|dimension| !(1..=MAXIMUM_DIMENSION as i32).contains(dimension))
	{
		return Err(invalid("SFBP dimensions must be between 1 and 65536"));
	}

	let content = std::str::from_utf8(&data)
		.map_err(|_| invalid("SFBP blueprints must be UTF-8 text"))?;
	let mut lines = content.lines();
	if lines.next() != Some(FORMAT_HEADER) {
		return Err(invalid("File is not a ShroudForge SFBP V7 blueprint"));
	}
	if !matches!(lines.next(), Some("x" | "y" | "z")) {
		return Err(invalid("SFBP rotation axis must be x, y, or z"));
	}

	let extent = lines
		.next()
		.and_then(parse_triplet)
		.ok_or_else(|| invalid("SFBP extent must contain three finite numbers"))?;
	let kind = lines.next().ok_or_else(|| invalid("SFBP blueprint kind is missing"))?;
	if extent.iter().any(|value| *value < 0.0)
		|| (kind == "props" && extent.iter().any(|value| *value <= 0.0))
	{
		return Err(invalid("SFBP extent is outside the supported range"));
	}

	let embedded_dimensions = match kind {
		"voxel" => {
			let dimensions = lines
				.next()
				.and_then(parse_dimensions)
				.ok_or_else(|| invalid("SFBP voxel dimensions are invalid"))?;
			let offset = lines
				.next()
				.and_then(parse_triplet)
				.ok_or_else(|| invalid("SFBP voxel offset is invalid"))?;
			let cell_size = lines
				.next()
				.and_then(parse_triplet)
				.ok_or_else(|| invalid("SFBP voxel cell size is invalid"))?;
			let cells = lines
				.next()
				.ok_or_else(|| invalid("SFBP voxel cells are missing"))?;
			let coverage = lines
				.next()
				.ok_or_else(|| invalid("SFBP voxel coverage is missing"))?;
			validate_voxel_channel(
				dimensions,
				offset,
				cell_size,
				cells,
				coverage,
			)
			.ok_or_else(|| invalid("SFBP voxel channel data is invalid"))?
		}
		"props" => {
			if lines.next() != Some("-")
				|| lines.next() != Some("-")
				|| lines.next() != Some("-")
				|| lines.next() != Some("-")
				|| lines.next() != Some("-")
			{
				return Err(invalid("Props-only SFBP must not contain voxel data"));
			}
			extent.map(|value| value.ceil() as u32)
		}
		_ => return Err(invalid("SFBP blueprint kind must be voxel or props")),
	};

	if embedded_dimensions
		.iter()
		.zip(metadata_dimensions)
		.any(|(embedded, metadata)| i32::try_from(*embedded).ok() != Some(metadata))
	{
		return Err(invalid(
			"SFBP file dimensions do not match the published blueprint dimensions",
		));
	}

	let prop_count = lines
		.next()
		.and_then(|value| value.parse::<usize>().ok())
		.filter(|count| *count <= MAXIMUM_PROPS)
		.ok_or_else(|| invalid("SFBP prop count is invalid or exceeds one million"))?;
	if !validate_props(lines, prop_count) {
		return Err(invalid("SFBP contains invalid prop data or an invalid row count"));
	}

	Ok(ValidationResult::Pass)
}
