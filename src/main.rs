use anyhow::{anyhow, Result};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;
use teloxide::prelude::*;

// =============================================================================
// MODELS
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DexScreenerResponse {
    pub pairs: Vec<Pair>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub chain_id: String,
    pub dex_id: String,
    pub pair_address: String,
    pub base_token: Token,
    pub quote_token: Token,
    pub price_usd: Option<String>,
    pub txns: TransactionStats,
    pub volume: VolumeStats,
    pub price_change: PriceChangeStats,
    pub liquidity: Option<LiquidityStats>,
    pub fdv: Option<f64>,
    pub market_cap: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionStats {
    pub h24: TxFrames,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxFrames {
    pub buys: u32,
    pub sells: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeStats {
    pub h24: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceChangeStats {
    pub m5: Option<f64>,
    pub h1: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LiquidityStats {
    pub usd: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RugCheckResponse {
    pub status: String,
    pub file_meta: Option<FileMeta>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMeta {
    pub bundle_ratio: Option<f64>,
}

// =============================================================================
// CONFIGURATION (Pure Rust)
// =============================================================================

pub struct Config {
    pub queries: Vec<String>,
    pub filters: Filters,
    pub blacklist: Blacklist,
    pub telegram: TelegramConfig,
}

pub struct Filters {
    pub min_liquidity_usd: f64,
    pub min_volume_h24_usd: f64,
    pub min_mcap_usd: f64,
    pub max_vlr: f64,
    pub max_bundled_supply_percent: f64,
}

pub struct Blacklist {
    pub tokens: Vec<String>,
}

pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
    pub bonkbot_ref: String, // e.g. "my_ref_code"
}

impl Config {
    pub fn new() -> Self {
        Self {
            queries: vec![
                "pump".to_string(),
                "pepe".to_string(),
                "solana".to_string(),
                "moon".to_string(),
            ],
            filters: Filters {
                min_liquidity_usd: 1000.0,
                min_volume_h24_usd: 5000.0,
                min_mcap_usd: 10000.0,
                max_vlr: 50.0,
                max_bundled_supply_percent: 25.0,
            },
            blacklist: Blacklist {
                tokens: vec!["0x0000000000000000000000000000000000000000".to_string()],
            },
            telegram: TelegramConfig {
                bot_token: "YOUR_TELEGRAM_BOT_TOKEN".to_string(),
                chat_id: "YOUR_CHAT_ID".to_string(),
                bonkbot_ref: "ref_code".to_string(),
            },
        }
    }
}

// =============================================================================
// ANALYSIS ENGINE
// =============================================================================

#[derive(Debug, PartialEq, Clone)]
pub enum MarketPattern {
    RugCandidate,
    PumpCandidate,
    StableTier1,
    FakeVolume,
    Blacklisted,
    RugcheckRisk,
    BundledSupply,
    Unknown,
}

pub struct AnalysisEngine;

impl AnalysisEngine {
    pub fn analyze_pair(pair: &Pair, config: &Config, rug_report: Option<&RugCheckResponse>) -> MarketPattern {
        if config.blacklist.tokens.contains(&pair.pair_address) {
            return MarketPattern::Blacklisted;
        }

        if let Some(report) = rug_report {
            if report.status != "good" {
                return MarketPattern::RugcheckRisk;
            }

            if let Some(meta) = &report.file_meta {
                if let Some(ratio) = meta.bundle_ratio {
                    if ratio * 100.0 > config.filters.max_bundled_supply_percent {
                        return MarketPattern::BundledSupply;
                    }
                }
            }
        }

        if let Some(liq) = &pair.liquidity {
            if let Some(liq_usd) = liq.usd {
                if liq_usd > 0.0 {
                    let vlr = pair.volume.h24 / liq_usd;
                    if vlr > config.filters.max_vlr {
                        return MarketPattern::FakeVolume;
                    }
                }
                if liq_usd < config.filters.min_liquidity_usd {
                    return MarketPattern::Unknown;
                }
            }
        }

        if pair.volume.h24 < config.filters.min_volume_h24_usd {
            return MarketPattern::Unknown;
        }

        if let Some(mcap) = pair.market_cap {
            if mcap < config.filters.min_mcap_usd {
                return MarketPattern::Unknown;
            }
        }

        if let Some(m5_change) = pair.price_change.m5 {
            if m5_change > 20.0 {
                return MarketPattern::PumpCandidate;
            }
            if m5_change < -50.0 {
                return MarketPattern::RugCandidate;
            }
        }

        MarketPattern::Unknown
    }
}

// =============================================================================
// MAIN EXECUTION LOOP
// =============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    println!("🚀 Starting Consolidated DexBot v4.0...");

    let mut config = Config::new();
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    
    // Telegram Bot Instance
    // Note: We use the token from config, but for real use, set it in environment.
    let bot = Bot::new(&config.telegram.bot_token);

    loop {
        for query in &config.queries {
            println!("🔍 Scanning: {}...", query);
            
            let dex_url = format!("https://api.dexscreener.com/latest/dex/search?q={}", query);
            if let Ok(resp) = client.get(&dex_url).send().await {
                if let Ok(data) = resp.json::<DexScreenerResponse>().await {
                    for pair in data.pairs {
                        if config.blacklist.tokens.contains(&pair.pair_address) { continue; }

                        // 1. Rugcheck
                        let rug_url = format!("https://api.rugcheck.xyz/v1/tokens/{}/report", pair.base_token.address);
                        let rug_report = if let Ok(r) = client.get(&rug_url).send().await {
                            r.json::<RugCheckResponse>().await.ok()
                        } else { None };

                        // 2. Analysis
                        let pattern = AnalysisEngine::analyze_pair(&pair, &config, rug_report.as_ref());

                        // 3. Auto-Blacklist & Notification
                        match &pattern {
                            MarketPattern::RugcheckRisk | MarketPattern::BundledSupply => {
                                println!("⛔ Security Risk: {} - Blacklisting.", pair.base_token.symbol);
                                config.blacklist.tokens.push(pair.pair_address.clone());
                            }
                            MarketPattern::PumpCandidate | MarketPattern::StableTier1 | MarketPattern::Unknown => {
                                // Only notify if it passed basic security and is a potential trade
                                if let Some(liq) = &pair.liquidity {
                                    if liq.usd.unwrap_or(0.0) >= config.filters.min_liquidity_usd && (pattern != MarketPattern::Unknown || pair.volume.h24 > config.filters.min_volume_h24_usd) {
                                        
                                        println!("✅ SIGNAL: {} found.", pair.base_token.symbol);
                                        
                                        let bonk_link = format!("https://t.me/bonkbot_bot?start={}_{}", config.telegram.bonkbot_ref, pair.base_token.address);
                                        let msg = format!(
                                            "💎 *NEW SIGNAL: {} ({})*\n\n💰 Mcap: ${:?}\n💧 Liq: ${:?}\n📈 Vol: ${:.2}\n\n[🚀 OPEN IN BONKBOT]({})",
                                            pair.base_token.name, pair.base_token.symbol, pair.market_cap, liq.usd, pair.volume.h24, bonk_link
                                        );

                                        let chat_id = ChatId(config.telegram.chat_id.parse().unwrap_or(0));
                                        if chat_id.0 != 0 {
                                            let _ = bot.send_message(chat_id, msg).parse_mode(teloxide::types::ParseMode::MarkdownV2).await;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }

                        // 4. Storage
                        let record = json!({
                            "ts": Utc::now().timestamp(),
                            "addr": pair.pair_address,
                            "sym": pair.base_token.symbol,
                            "price": pair.price_usd,
                            "pattern": format!("{:?}", pattern)
                        });
                        let mut file = OpenOptions::new().create(true).append(true).open("dex_data.jsonl")?;
                        writeln!(file, "{}", record.to_string())?;
                    }
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
        println!("Cycle complete. Waiting 60s...");
        sleep(Duration::from_secs(60)).await;
    }
}
