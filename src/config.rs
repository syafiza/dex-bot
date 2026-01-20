pub struct Config {
    pub filters: Filters,
    pub blacklist: Blacklist,
    pub queries: Vec<String>,
    pub rugcheck: RugcheckConfig,
}

pub struct Filters {
    pub min_liquidity_usd: f64,
    pub min_volume_h24_usd: f64,
    pub min_mcap_usd: f64,
    pub max_vlr: f64, // Volume to Liquidity Ratio
    pub max_bundled_supply_percent: f64,
}

pub struct Blacklist {
    pub tokens: Vec<String>,
    pub devs: Vec<String>,
}

pub struct RugcheckConfig {
    pub api_key: Option<String>,
    pub only_good: bool,
}

impl Config {
    /// Pure Rust configuration initialization.
    /// Professional setups often prefer compile-time or code-defined defaults for critical bot logic.
    pub fn new() -> Self {
        Self {
            filters: Filters {
                min_liquidity_usd: 1000.0,
                min_volume_h24_usd: 5000.0,
                min_mcap_usd: 10000.0,
                max_vlr: 50.0,
                max_bundled_supply_percent: 25.0, // Flag if top holders have > 25%
            },
            blacklist: Blacklist {
                tokens: vec![
                    "0x0000000000000000000000000000000000000000".to_string(),
                ],
                devs: vec![
                    "rugged_dev_id_1".to_string(),
                ],
            },
            queries: vec![
                "pump".to_string(),
                "pepe".to_string(),
                "solana".to_string(),
                "moon".to_string(),
            ],
            rugcheck: RugcheckConfig {
                api_key: None,
                only_good: true,
            },
        }
    }
}
