use anyhow::{anyhow, Result};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tokio::sync::Mutex;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveTrade {
    pub address: String,
    pub symbol: String,
    pub entry_price: f64,
    pub amount_sol: f64,
    pub entry_time: i64,
}

// =============================================================================
// CONFIGURATION (Pure Rust)
// =============================================================================

pub struct Config {
    pub queries: Vec<String>,
    pub filters: Filters,
    pub blacklist: Blacklist,
    pub telegram: TelegramConfig,
    pub paper_trading: PaperTradingConfig,
    pub database: DatabaseConfig,
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
    pub bonkbot_ref: String,
}

pub struct PaperTradingConfig {
    pub enabled: bool,
    pub buy_amount_sol: f64,
    pub take_profit_percent: f64,
    pub stop_loss_percent: f64,
}

pub struct DatabaseConfig {
    pub url: String,
}

impl Config {
    pub fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost/dexbot".to_string());
        Self {
            queries: vec![
                "pump".to_string(),
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
                bot_token: std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default(),
                chat_id: std::env::var("TELEGRAM_CHAT_ID").unwrap_or_default(),
                bonkbot_ref: "ref_code".to_string(),
            },
            paper_trading: PaperTradingConfig {
                enabled: true,
                buy_amount_sol: 0.1,
                take_profit_percent: 50.0,
                stop_loss_percent: 25.0,
            },
            database: DatabaseConfig {
                url: db_url,
            },
        }
    }
}

pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;

        // Initialize Schema
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS scans (
                id SERIAL PRIMARY KEY,
                ts TIMESTAMPTZ NOT NULL,
                address TEXT NOT NULL,
                symbol TEXT NOT NULL,
                pattern TEXT NOT NULL,
                data JSONB NOT NULL
            );
            CREATE TABLE IF NOT EXISTS active_trades (
                id SERIAL PRIMARY KEY,
                address TEXT NOT NULL UNIQUE,
                symbol TEXT NOT NULL,
                entry_price DOUBLE PRECISION NOT NULL,
                amount_sol DOUBLE PRECISION NOT NULL,
                entry_time TIMESTAMPTZ NOT NULL
            );
            CREATE TABLE IF NOT EXISTS historical_trades (
                id SERIAL PRIMARY KEY,
                address TEXT NOT NULL,
                symbol TEXT NOT NULL,
                entry_price DOUBLE PRECISION NOT NULL,
                exit_price DOUBLE PRECISION NOT NULL,
                pnl_percent DOUBLE PRECISION NOT NULL,
                entry_time TIMESTAMPTZ NOT NULL,
                exit_time TIMESTAMPTZ NOT NULL
            );"
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn log_scan(&self, pair: &Pair, pattern: MarketPattern) -> Result<()> {
        sqlx::query(
            "INSERT INTO scans (ts, address, symbol, pattern, data) VALUES (NOW(), $1, $2, $3, $4)"
        )
        .bind(&pair.pair_address)
        .bind(&pair.base_token.symbol)
        .bind(format!("{:?}", pattern))
        .bind(json!(pair))
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// =============================================================================
// ANALYSIS ENGINE
// =============================================================================

#[derive(Debug, PartialEq, Clone)]
pub enum MarketPattern {
    GoodCandidate,
    RugCandidate,
    PumpCandidate,
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

        // Professional Signal detection
        if let Some(m5) = pair.price_change.m5 {
            if m5 > 5.0 && m5 < 50.0 { // Healthy growth
                return MarketPattern::GoodCandidate;
            }
        }

        MarketPattern::Unknown
    }
}

// =============================================================================
// PAPER TRADING ENGINE
// =============================================================================

pub struct PaperTradingEngine {
    pub active_trades: Arc<Mutex<Vec<ActiveTrade>>>,
}

impl PaperTradingEngine {
    pub fn new() -> Self {
        Self {
            active_trades: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn process_signal(&self, pair: &Pair, config: &Config) {
        let mut trades = self.active_trades.lock().await;
        if trades.iter().any(|t| t.address == pair.base_token.address) {
            return; // Already in trade
        }

        if let Some(price_str) = &pair.price_usd {
            if let Ok(price) = price_str.parse::<f64>() {
                let trade = ActiveTrade {
                    address: pair.base_token.address.clone(),
                    symbol: pair.base_token.symbol.clone(),
                    entry_price: price,
                    amount_sol: config.paper_trading.buy_amount_sol,
                    entry_time: Utc::now().timestamp(),
                };
                trades.push(trade);
                println!("📈 [PAPER TRADE] ENTER: {} at ${:.8}", pair.base_token.symbol, price);
            }
        }
    }

    pub async fn monitor_trades(&self, client: &Client, config: &Config, bot: &Bot) {
        let mut trades = self.active_trades.lock().await;
        let mut to_remove = Vec::new();

        for (idx, trade) in trades.iter().enumerate() {
            let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", trade.address);
            if let Ok(resp) = client.get(&url).send().await {
                if let Ok(data) = resp.json::<DexScreenerResponse>().await {
                    if let Some(pair) = data.pairs.first() {
                        if let Some(price_str) = &pair.price_usd {
                            if let Ok(current_price) = price_str.parse::<f64>() {
                                let pnl = ((current_price - trade.entry_price) / trade.entry_price) * 100.0;
                                
                                println!("📊 [PAPER TRADE] {} PnL: {:.2}%", trade.symbol, pnl);

                                if pnl >= config.paper_trading.take_profit_percent || pnl <= -config.paper_trading.stop_loss_percent {
                                    println!("📉 [PAPER TRADE] EXIT: {} at {:.2}% PnL", trade.symbol, pnl);
                                    
                                    // Notify via Telegram
                                    let msg = format!(
                                        "🔔 *PAPER TRADE CLOSED*\n\nToken: {}\nResult: {:.2}%\nExit Price: ${:.8}",
                                        trade.symbol, pnl, current_price
                                    );
                                    let chat_id = ChatId(config.telegram.chat_id.parse().unwrap_or(0));
                                    if chat_id.0 != 0 {
                                        let _ = bot.send_message(chat_id, msg).parse_mode(teloxide::types::ParseMode::MarkdownV2).await;
                                    }

                                    to_remove.push(idx);
                                }
                            }
                        }
                    }
                }
            }
            sleep(Duration::from_millis(500)).await; // Prevent rate limit
        }

        for idx in to_remove.into_iter().rev() {
            trades.remove(idx);
        }
    }
}

// =============================================================================
// BACKTESTING ENGINE
// =============================================================================

pub struct BacktestingEngine;

impl BacktestingEngine {
    pub async fn run(config: &Config) -> Result<()> {
        println!("🧪 Starting Backtest from dex_data.jsonl...");
        let file = std::fs::read_to_string("dex_data.jsonl")?;
        let mut good_signals = 0;
        let mut total_scans = 0;

        for line in file.lines() {
            if let Ok(record) = serde_json::from_str::<serde_json::Value>(line) {
                total_scans += 1;
                // Note: Simplified for backtest (does not re-verify Rugcheck in replay)
                if record["pattern"] == "GoodCandidate" {
                    good_signals += 1;
                }
            }
        }

        println!("📊 BACKTEST RESULTS:");
        println!("Total Scans: {}", total_scans);
        println!("Good Signals identified: {}", good_signals);
        if total_scans > 0 {
            println!("Signal Ratio: {:.2}%", (good_signals as f64 / total_scans as f64) * 100.0);
        }
        Ok(())
    }
}

// =============================================================================
// MAIN EXECUTION LOOP
// =============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();
    
    let args: Vec<String> = std::env::args().collect();
    let mut config = Config::new();

    if args.contains(&"--backtest".to_string()) {
        BacktestingEngine::run(&config).await?;
        return Ok(());
    }

    println!("🚀 Starting Consolidated DexBot v6.0 (PostgreSQL Active)...");

    // Initialize Database
    let db = match Database::new(&config.database.url).await {
        Ok(d) => {
            println!("✅ PostgreSQL Connected.");
            Some(d)
        }
        Err(e) => {
            eprintln!("⚠️ Database connection failed: {}. Falling back to JSONL only.", e);
            None
        }
    };

    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let bot = Bot::new(&config.telegram.bot_token);
    let paper_engine = PaperTradingEngine::new();

    // Spawn monitoring task
    let pe_mon = Arc::new(paper_engine);
    let pe_mon_clone = Arc::clone(&pe_mon);
    let client_clone = client.clone();
    let bot_clone = bot.clone();
    
    tokio::spawn(async move {
        loop {
            let mon_config = Config::new(); 
            pe_mon_clone.monitor_trades(&client_clone, &mon_config, &bot_clone).await;
            sleep(Duration::from_secs(30)).await;
        }
    });

    loop {
        for query in &config.queries {
            println!("🔍 Scanning: {}...", query);
            
            let dex_url = format!("https://api.dexscreener.com/latest/dex/search?q={}", query);
            if let Ok(resp) = client.get(&dex_url).send().await {
                if let Ok(data) = resp.json::<DexScreenerResponse>().await {
                    for pair in data.pairs {
                        if config.blacklist.tokens.contains(&pair.pair_address) { continue; }

                        let rug_url = format!("https://api.rugcheck.xyz/v1/tokens/{}/report", pair.base_token.address);
                        let rug_report = if let Ok(r) = client.get(&rug_url).send().await {
                            r.json::<RugCheckResponse>().await.ok()
                        } else { None };

                        let pattern = AnalysisEngine::analyze_pair(&pair, &config, rug_report.as_ref());

                        // Log to PG if connected
                        if let Some(ref d) = db {
                            let _ = d.log_scan(&pair, pattern.clone()).await;
                        }

                        match &pattern {
                            MarketPattern::RugcheckRisk | MarketPattern::BundledSupply => {
                                println!("⛔ Security Risk: {} - Blacklisting.", pair.base_token.symbol);
                                config.blacklist.tokens.push(pair.pair_address.clone());
                            }
                            MarketPattern::GoodCandidate => {
                                println!("✅ SIGNAL: {} found.", pair.base_token.symbol);
                                
                                if config.paper_trading.enabled {
                                    pe_mon.process_signal(&pair, &config).await;
                                }

                                let bonk_link = format!("https://t.me/bonkbot_bot?start={}_{}", config.telegram.bonkbot_ref, pair.base_token.address);
                                let msg = format!(
                                    "💎 *GOOD SIGNAL: {} ({})*\n\n💰 Mcap: ${:?}\n💧 Liq: ${:?}\n📈 Vol: ${:.2}\n\n[🚀 OPEN IN BONKBOT]({})",
                                    pair.base_token.name, pair.base_token.symbol, pair.market_cap, pair.liquidity.as_ref().and_then(|l| l.usd), pair.volume.h24, bonk_link
                                );

                                let chat_id = ChatId(config.telegram.chat_id.parse().unwrap_or(0));
                                if chat_id.0 != 0 {
                                    let _ = bot.send_message(chat_id, msg).parse_mode(teloxide::types::ParseMode::MarkdownV2).await;
                                }
                            }
                            _ => {}
                        }

                        // Local JSONL Backup
                        let record = json!({
                            "ts": Utc::now().timestamp(),
                            "addr": pair.pair_address,
                            "sym": pair.base_token.symbol,
                            "pattern": format!("{:?}", pattern)
                        });
                        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("dex_data.jsonl") {
                            let _ = writeln!(file, "{}", record.to_string());
                        }
                    }
                }
            }
            sleep(Duration::from_secs(10)).await;
        }
        println!("Cycle complete. Waiting 60s...");
        sleep(Duration::from_secs(60)).await;
    }
}
