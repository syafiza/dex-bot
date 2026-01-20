use crate::models::DexScreenerResponse;
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::time::Duration;

pub struct DexScreenerClient {
    client: Client,
}

impl DexScreenerClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Self { client }
    }

    pub async fn search_pairs(&self, query: &str) -> Result<DexScreenerResponse> {
        let url = format!("https://api.dexscreener.com/latest/dex/search?q={}", query);
        let resp = self.client.get(&url).send().await?;
        
        if resp.status().is_success() {
            let data = resp.json::<DexScreenerResponse>().await?;
            Ok(data)
        } else {
            Err(anyhow!("Failed to fetch search results: {}", resp.status()))
        }
    }

    pub async fn get_token_pairs(&self, token_address: &str) -> Result<DexScreenerResponse> {
        let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", token_address);
        let resp = self.client.get(&url).send().await?;
        
        if resp.status().is_success() {
            let data = resp.json::<DexScreenerResponse>().await?;
            Ok(data)
        } else {
            Err(anyhow!("Failed to fetch token pairs: {}", resp.status()))
        }
    }
}
