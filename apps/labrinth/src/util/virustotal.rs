use bytes::Bytes;
use eyre::{Result, WrapErr, eyre};
use reqwest::{StatusCode, multipart};
use serde_json::Value;

use crate::{env::ENV, util::http::HTTP_CLIENT};

const API_URL: &str = "https://www.virustotal.com/api/v3";

#[derive(Clone)]
pub struct VirusTotalClient {
	api_key: String,
}

impl VirusTotalClient {
	pub fn new() -> Result<Self> {
		if ENV.VIRUSTOTAL_API_KEY.is_empty() {
			return Err(eyre!("VirusTotal API key is not configured"));
		}

		Ok(Self {
			api_key: ENV.VIRUSTOTAL_API_KEY.clone(),
		})
	}

	pub async fn get_upload_url(&self) -> Result<String> {
		let response = self
			.request(HTTP_CLIENT.get(format!("{API_URL}/files/upload_url")))
			.await?;
		let value = response
			.json::<Value>()
			.await
			.wrap_err("reading VirusTotal upload URL response")?;
		value["data"]
			.as_str()
			.map(ToOwned::to_owned)
			.ok_or_else(|| eyre!("VirusTotal upload URL response has no URL"))
	}

	pub async fn upload_file(
		&self,
		upload_url: Option<&str>,
		file_name: &str,
		file_data: Bytes,
	) -> Result<String> {
		let url = upload_url
			.map(ToOwned::to_owned)
			.unwrap_or_else(|| format!("{API_URL}/files"));
		let part = multipart::Part::bytes(file_data.to_vec())
			.file_name(file_name.to_string());
		let response = self
			.request(
				HTTP_CLIENT
					.post(url)
					.multipart(multipart::Form::new().part("file", part)),
			)
			.await?;
		let value = response
			.json::<Value>()
			.await
			.wrap_err("reading VirusTotal upload response")?;
		value["data"]["id"]
			.as_str()
			.map(ToOwned::to_owned)
			.ok_or_else(|| {
				eyre!("VirusTotal upload response has no analysis ID")
			})
	}

	pub async fn get_analysis(&self, analysis_id: &str) -> Result<Value> {
		let response = self
			.request(
				HTTP_CLIENT.get(format!("{API_URL}/analyses/{analysis_id}")),
			)
			.await?;
		let value = response
			.json::<Value>()
			.await
			.wrap_err("reading VirusTotal analysis response")?;
		value
			.get("data")
			.and_then(|data| data.get("attributes"))
			.cloned()
			.ok_or_else(|| {
				eyre!("VirusTotal analysis response has no attributes")
			})
	}

	async fn request(
		&self,
		request: reqwest::RequestBuilder,
	) -> Result<reqwest::Response> {
		let response = request
			.header("x-apikey", &self.api_key)
			.send()
			.await
			.wrap_err("sending VirusTotal request")?;
		if response.status() == StatusCode::TOO_MANY_REQUESTS {
			return Err(eyre!("VirusTotal request rate limit reached"));
		}
		response
			.error_for_status()
			.wrap_err("VirusTotal request failed")
	}
}
