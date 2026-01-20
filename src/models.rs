use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DexScreenerResponse {
    pub pairs: Vec<Pair>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub chain_id: String,
    pub dex_id: String,
    pub url: String,
    pub pair_address: String,
    pub base_token: Token,
    pub quote_token: Token,
    pub price_native: String,
    pub price_usd: Option<String>,
    pub txns: TransactionStats,
    pub volume: VolumeStats,
    pub price_change: PriceChangeStats,
    pub liquidity: Option<LiquidityStats>,
    pub fdv: Option<f64>,
    pub market_cap: Option<f64>,
    pub pair_created_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionStats {
    pub m5: TxFrames,
    pub h1: TxFrames,
    pub h6: TxFrames,
    pub h24: TxFrames,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxFrames {
    pub buys: u32,
    pub sells: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeStats {
    pub m5: f64,
    pub h1: f64,
    pub h6: f64,
    pub h24: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceChangeStats {
    pub m5: Option<f64>,
    pub h1: Option<f64>,
    pub h6: Option<f64>,
    pub h24: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LiquidityStats {
    pub usd: Option<f64>,
    pub base: f64,
    pub quote: f64,
}
