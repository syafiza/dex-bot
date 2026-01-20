use crate::models::Pair;
use crate::config::Config;
use crate::rugcheck::RugCheckResponse;

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
        // 1. Check Blacklist
        if config.blacklist.tokens.contains(&pair.pair_address) {
            return MarketPattern::Blacklisted;
        }

        // 2. Rugcheck Security Check
        if let Some(report) = rug_report {
            if config.rugcheck.only_good && report.status != "good" {
                return MarketPattern::RugcheckRisk;
            }

            // 3. Bundled Supply Detection
            // Using rugcheck's bundle ratio if available, or analyzing top holder concentrations
            if let Some(meta) = &report.file_meta {
                if let Some(ratio) = meta.bundle_ratio {
                    if ratio * 100.0 > config.filters.max_bundled_supply_percent {
                        return MarketPattern::BundledSupply;
                    }
                }
            }
        }

        // 4. Fake Volume Detection (Wash Trading)
        if let Some(liq) = &pair.liquidity {
            if let Some(liq_usd) = liq.usd {
                if liq_usd > 0.0 {
                    let vlr = pair.volume.h24 / liq_usd;
                    if vlr > config.filters.max_vlr {
                        return MarketPattern::FakeVolume;
                    }
                }
            }
        }

        // 5. Apply Advanced Filters from Config
        if let Some(liq) = &pair.liquidity {
            if let Some(liq_usd) = liq.usd {
                if liq_usd < config.filters.min_liquidity_usd {
                    return MarketPattern::Unknown;
                }
            } else {
                return MarketPattern::Unknown;
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

        // 6. Pattern Heuristics
        if let Some(m5_change) = pair.price_change.m5 {
            if m5_change < -50.0 {
                return MarketPattern::RugCandidate;
            }
        }

        if let Some(m5_change) = pair.price_change.m5 {
            if m5_change > 20.0 {
                return MarketPattern::PumpCandidate;
            }
        }

        if let Some(mcap) = pair.market_cap {
            if mcap > 10_000_000.0 {
                if let Some(liq) = &pair.liquidity {
                    if let Some(liq_usd) = liq.usd {
                        if liq_usd > 500_000.0 && pair.volume.h24 > 1_000_000.0 {
                            return MarketPattern::StableTier1;
                        }
                    }
                }
            }
        }

        MarketPattern::Unknown
    }
}
