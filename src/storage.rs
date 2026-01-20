use anyhow::Result;
use crate::models::Pair;
use chrono::Utc;
use std::fs::{OpenOptions, File};
use std::io::Write;
use serde_json::json;

pub struct Database {
    file_path: String,
}

impl Database {
    pub async fn new(path: &str) -> Result<Self> {
        // Just verify/create the file
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
            
        Ok(Self { file_path: path.to_string() })
    }

    pub async fn save_pair(&self, pair: &Pair) -> Result<()> {
        let now = Utc::now().timestamp();
        
        let record = json!({
            "timestamp": now,
            "address": pair.pair_address,
            "chain_id": pair.chain_id,
            "base": pair.base_token.symbol,
            "price_usd": pair.price_usd,
            "liquidity": pair.liquidity.as_ref().and_then(|l| l.usd),
            "volume_h24": pair.volume.h24,
            "buys": pair.txns.h24.buys,
            "sells": pair.txns.h24.sells,
        });

        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.file_path)?;

        writeln!(file, "{}", record.to_string())?;

        Ok(())
    }
}
