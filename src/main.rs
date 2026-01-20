mod models;
mod client;
mod storage;
mod analysis;
mod config;
mod rugcheck;

use client::DexScreenerClient;
use storage::Database;
use analysis::{AnalysisEngine, MarketPattern};
use config::Config;
use rugcheck::RugCheckClient;
use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    dotenv::dotenv().ok();

    println!("🚀 Starting DexScreener Analysis Bot v3.0 (Security Focus)...");

    // Initialize Pure Rust configuration
    // In a real scenario, this could be loaded from an environment or a more complex source,
    // but for 100% Rust we define it in config.rs.
    let mut config = Config::new();
    println!("✅ Internal config initialized.");

    // Initialize database
    let db = Database::new("dex_data.jsonl").await?;
    println!("📦 JSON storage initialized (dex_data.jsonl).");

    // Initialize clients
    let dex_client = DexScreenerClient::new();
    let rug_client = RugCheckClient::new();

    loop {
        for query in &config.queries {
            println!("🔍 Scanning for: {}...", query);
            
            match dex_client.search_pairs(query).await {
                Ok(resp) => {
                    for pair in resp.pairs {
                        // 1. Core Blacklist Check
                        if config.blacklist.tokens.contains(&pair.pair_address) {
                            continue;
                        }

                        // 2. Perform Rugcheck Scan
                        println!("🛡️ Checking security for {}...", pair.base_token.symbol);
                        let rug_report = match rug_client.scan_token(&pair.base_token.address).await {
                            Ok(report) => Some(report),
                            Err(e) => {
                                eprintln!("⚠️ Rugcheck failed for {}: {}", pair.base_token.symbol, e);
                                None
                            }
                        };

                        // 3. Analyze pair with Rugcheck info
                        let pattern = AnalysisEngine::analyze_pair(&pair, &config, rug_report.as_ref());
                        
                        // 4. Handle Security Risks (Auto-Blacklist)
                        match pattern {
                            MarketPattern::RugcheckRisk | MarketPattern::BundledSupply => {
                                println!("⛔ SECURITY ALERT: {} ({}) - Auto-blacklisting.", pair.base_token.name, pair.base_token.symbol);
                                config.blacklist.tokens.push(pair.pair_address.clone());
                                // In a real scenario, we might also blacklist the dev address if available in report
                            }
                            _ => {}
                        }

                        // 5. Save to DB
                        if let Err(e) = db.save_pair(&pair).await {
                            eprintln!("Error saving pair: {}", e);
                        }

                        // 6. Report findings
                        match pattern {
                            MarketPattern::RugCandidate => {
                                println!("⚠️  RUG RISK: {} ({}) on {}", pair.base_token.name, pair.base_token.symbol, pair.chain_id);
                            }
                            MarketPattern::PumpCandidate => {
                                println!("🔥 PUMP: {} ({}) +{:?}% in 5m", pair.base_token.name, pair.base_token.symbol, pair.price_change.m5);
                            }
                            MarketPattern::StableTier1 => {
                                println!("💎 TIER-1: {} ({}) - Mcap: ${:?}", pair.base_token.name, pair.base_token.symbol, pair.market_cap);
                            }
                            MarketPattern::FakeVolume => {
                                println!("🚫 FAKE VOLUME: {} ({}) - Skipping.", pair.base_token.name, pair.base_token.symbol);
                            }
                            MarketPattern::RugcheckRisk => {
                                println!("🚫 RUGCHECK REJECTED: {} ({})", pair.base_token.name, pair.base_token.symbol);
                            }
                            MarketPattern::BundledSupply => {
                                println!("🚫 BUNDLED SUPPLY: {} ({})", pair.base_token.name, pair.base_token.symbol);
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => eprintln!("Error searching pairs for {}: {}", query, e),
            }

            // Sleep between queries to avoid rate limits
            sleep(Duration::from_secs(5)).await;
        }

        println!("Cycle complete. Waiting 60s...");
        sleep(Duration::from_secs(60)).await;
    }
}
