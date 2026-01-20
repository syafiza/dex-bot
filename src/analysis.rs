use crate::models::Pair;

#[derive(Debug, PartialEq)]
pub enum MarketPattern {
    RugCandidate,
    PumpCandidate,
    StableTier1,
    Unknown,
}

pub struct AnalysisEngine;

impl AnalysisEngine {
    pub fn analyze_pair(pair: &Pair) -> MarketPattern {
        // Simple heuristics for demonstration
        
        // 1. Check for Rug Candidates (Low liquidity, high FDV ratio, or massive price drop)
        if let Some(liq) = &pair.liquidity {
            if let Some(liq_usd) = liq.usd {
                if liq_usd < 1000.0 && pair.volume.h24 > 10000.0 {
                    return MarketPattern::RugCandidate;
                }
            }
        }

        // 2. Check for Pump Candidates (High 5m or 1h price change)
        if let Some(m5_change) = pair.price_change.m5 {
            if m5_change > 20.0 {
                return MarketPattern::PumpCandidate;
            }
        }
        if let Some(h1_change) = pair.price_change.h1 {
            if h1_change > 50.0 {
                return MarketPattern::PumpCandidate;
            }
        }

        // 3. Check for Tier-1 Candidates (High volume, high liquidity, consistent growth)
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
