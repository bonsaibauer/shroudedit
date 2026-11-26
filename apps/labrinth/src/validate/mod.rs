use crate::database::PgTransaction;
use crate::models::projects::{FileType, Loader};
use bytes::Bytes;
use thiserror::Error;
use xredis::RedisPool;

mod schematic;
mod schematic_v8;
pub mod sfpack;
pub mod shroudforge;

#[derive(Error, Debug)]
pub enum ValidationError {
	#[error("Unable to read Zip Archive: {0}")]
	Zip(#[from] zip::result::ZipError),
	#[error("IO Error: {0}")]
	Io(#[from] std::io::Error),
	#[error("Error while validating JSON for uploaded file: {0}")]
	SerDe(#[from] serde_json::Error),
	#[error("Invalid Input: {0}")]
	InvalidInput(std::borrow::Cow<'static, str>),
	#[error("Error while managing threads")]
	Blocking(#[from] actix_web::error::BlockingError),
	#[error("internal error while validating uploaded file")]
	Internal(#[from] eyre::Report),
}

#[derive(Eq, PartialEq, Debug)]
pub enum ValidationResult {
	Pass,
	Warning(&'static str),
}

impl ValidationResult {
	pub fn is_passed(&self) -> bool {
		matches!(self, Self::Pass)
	}
}

pub async fn validate_file(
	data: Bytes,
	file_extension: String,
	loaders: Vec<Loader>,
	file_type: Option<FileType>,
	version_fields: Vec<crate::database::models::loader_fields::VersionField>,
	_transaction: &mut PgTransaction<'_>,
	_redis: &RedisPool,
) -> Result<ValidationResult, ValidationError> {
	if file_extension == "sfbp" {
		return schematic::validate_file(data, loaders, version_fields).await;
	}
	if file_extension == "sfpack" {
		if !loaders.iter().any(|loader| loader.0 == "sfpack") {
			return Err(ValidationError::InvalidInput(
				".sfpack requires the sfpack loader".into(),
			));
		}
		sfpack::validate_package(data, None, None)
			.map_err(|error| ValidationError::InvalidInput(error.into()))?;
		return Ok(ValidationResult::Pass);
	}
	if matches!(file_type, Some(FileType::Signature))
		&& !["asc", "gpg", "sig"].contains(&file_extension.as_str())
	{
		return Err(ValidationError::InvalidInput(
			format!(
				"File extension {file_extension} is invalid for input file"
			)
			.into(),
		));
	}
	Ok(ValidationResult::Pass)
}
