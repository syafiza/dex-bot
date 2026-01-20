pub struct Config {
    pub filters: Filters,
    pub blacklist: Blacklist,
    pub queries: Vec<String>,
}

pub struct Filters {
    pub min_liquidity_usd: f64,
    pub min_volume_h24_usd: f64,
    pub min_mcap_usd: f64,
    pub max_vlr: f64, // Volume to Liquidity Ratio
}

pub struct Blacklist {
    pub tokens: Vec<String>,
    pub devs: Vec<String>,
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
        }
    }
}
