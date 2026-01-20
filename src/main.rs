mod models;
mod client;
mod storage;
mod analysis;

use client::DexScreenerClient;
use storage::Database;
use analysis::{AnalysisEngine, MarketPattern};
use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    dotenv::dotenv().ok();

    println!("🚀 Starting DexScreener Analysis Bot...");

    // Initialize database
    let db = Database::new("dex_data.jsonl").await?;
    println!("JSON storage initialized (dex_data.jsonl).");

    // Initialize client
    let client = DexScreenerClient::new();

    // Queries to monitor
    let queries = vec!["pump", "pepe", "solana", "moon"];

    loop {
        for query in &queries {
            println!("🔍 Scanning for: {}...", query);
            
            match client.search_pairs(query).await {
                Ok(resp) => {
                    for pair in resp.pairs {
                        // Analyze pair
                        let pattern = AnalysisEngine::analyze_pair(&pair);
                        
                        // Save to DB
                        if let Err(e) = db.save_pair(&pair).await {
                            eprintln!("Error saving pair: {}", e);
                        }

                        // Report findings
                        match pattern {
                            MarketPattern::RugCandidate => {
                                println!("⚠️  RUG RISK DETECTED: {} ({}) on {}", pair.base_token.name, pair.base_token.symbol, pair.chain_id);
                            }
                            MarketPattern::PumpCandidate => {
                                println!("🔥 PUMP DETECTED: {} ({}) +{:?}% in 5m", pair.base_token.name, pair.base_token.symbol, pair.price_change.m5);
                            }
                            MarketPattern::StableTier1 => {
                                println!("💎 TIER-1 CANDIDATE: {} ({}) - Mcap: ${:?}", pair.base_token.name, pair.base_token.symbol, pair.market_cap);
                            }
                            MarketPattern::Unknown => {}
                        }
                    }
                }
                Err(e) => eprintln!("Error searching pairs for {}: {}", query, e),
            }

            // Sleep between queries to avoid rate limits
            sleep(Duration::from_secs(2)).await;
        }

        println!("Waiting for next scan cycle...");
        sleep(Duration::from_secs(60)).await;
    }
}
