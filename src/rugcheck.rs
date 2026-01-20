use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RugCheckResponse {
    pub score: i32,
    pub status: String,
    pub risks: Vec<Risk>,
    pub file_meta: Option<FileMeta>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Risk {
    pub name: String,
    pub value: String,
    pub description: String,
    pub score: i32,
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMeta {
    pub bundle_ratio: Option<f64>,
}

pub struct RugCheckClient {
    client: Client,
    api_url: String,
}

impl RugCheckClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Self {
            client,
            api_url: "https://api.rugcheck.xyz/v1".to_string(),
        }
    }

    pub async fn scan_token(&self, address: &str) -> Result<RugCheckResponse> {
        let url = format!("{}/tokens/{}/report", self.api_url, address);
        let resp = self.client.get(&url).send().await?;
        
        if resp.status().is_success() {
            let data = resp.json::<RugCheckResponse>().await?;
            Ok(data)
        } else {
            Err(anyhow!("Rugcheck API error: {}", resp.status()))
        }
    }
}
