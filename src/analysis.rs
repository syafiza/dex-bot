use crate::models::Pair;
use crate::config::Config;

#[derive(Debug, PartialEq, Clone)]
pub enum MarketPattern {
    RugCandidate,
    PumpCandidate,
    StableTier1,
    FakeVolume,
    Blacklisted,
    Unknown,
}

pub struct AnalysisEngine;

impl AnalysisEngine {
    pub fn analyze_pair(pair: &Pair, config: &Config) -> MarketPattern {
        // 1. Check Blacklist
        if config.blacklist.tokens.contains(&pair.pair_address) {
            return MarketPattern::Blacklisted;
        }
        
        // Note: Dexscreener doesn't always provide a specific 'Dev' address in the basic pair response,
        // but if it were in the token info, we would check it here. 
        // For now, we'll assume the user might blacklist specific base token addresses as 'dev' proxies.

        // 2. Fake Volume Detection (Wash Trading)
        // Heuristic: Volume/Liquidity Ratio (VLR). 
        // If volume is 50x higher than liquidity, it's highly likely to be wash trading.
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

        // 3. Apply Advanced Filters from Config
        if let Some(liq) = &pair.liquidity {
            if let Some(liq_usd) = liq.usd {
                if liq_usd < config.filters.min_liquidity_usd {
                    return MarketPattern::Unknown; // Filtered out
                }
            } else {
                return MarketPattern::Unknown; // No liquidity data
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

        // 4. Pattern Heuristics
        
        // Rug Candidates (Still useful as a secondary check)
        if let Some(m5_change) = pair.price_change.m5 {
            if m5_change < -50.0 {
                return MarketPattern::RugCandidate;
            }
        }

        // Pump Candidates
        if let Some(m5_change) = pair.price_change.m5 {
            if m5_change > 20.0 {
                return MarketPattern::PumpCandidate;
            }
        }

        // Tier-1 Candidates
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
