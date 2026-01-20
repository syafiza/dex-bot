# 🤖 DexScreener Analysis Bot (Pure Rust)

A high-performance, professional-grade Rust bot designed to interact with the DexScreener API. This tool parses, logs, and analyzes token pairs in real-time to identify market patterns such as Rug Risks, Pumps, and Tier-1 transitions.

Built with a focus on speed, safety, and 100% Rust portability.

## 🚀 Key Features

- **Real-Time Monitoring**: Polls DexScreener for the latest pairs and trending tokens based on custom search queries.
- **Fake Volume Detection**: Integrated heuristics to detect wash trading by analyzing Volume-to-Liquidity Ratios (VLR).
- **Pattern Analysis Engine**:
    - ⚠️ **Rug Risk**: Detects low liquidity vs abnormal volume/price drops.
    - 🔥 **Pump Candidate**: Identifies tokens with rapid short-term buy pressure.
    - 💎 **Tier-1 Potential**: Flags tokens with high market cap, consistent organic volume, and strong liquidity.
- **100% Pure Rust Configuration**: No external JSON/YAML dependencies. All filters, blacklists, and scan settings are managed natively in `src/config.rs`.
- **Persistent Metrics**: Logs all data to `dex_data.jsonl` for historical analysis and AI model training.
- **Async Architecture**: Powered by `tokio` for multi-threaded, non-blocking execution.

## 🛠 Project Structure

- `src/main.rs`: Core execution loop and orchestrator.
- `src/config.rs`: Professional-grade configuration (Filters, Blacklists, Queries).
- `src/analysis.rs`: Heuristic-based logic engine for market pattern detection.
- `src/client.rs`: Async API client for DexScreener.
- `src/models.rs`: Type-safe bindings for DexScreener API responses.
- `src/storage.rs`: High-performance JSONL persistence layer.

## 🏁 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)

### Installation
```bash
git clone https://github.com/syafiza/dex-bot.git
cd dex-bot
```

### Running the Bot
```bash
cargo run --release
```

## ⚙️ Configuration

To modify filters, add tokens to the blacklist, or change scan queries, edit [src/config.rs](file:///c:/Users/admin/OneDrive/Documents/AI%20Research/dex-bot/src/config.rs):

```rust
// Example config logic in src/config.rs
pub fn new() -> Self {
    Self {
        filters: Filters {
            min_liquidity_usd: 1000.0,
            max_vlr: 50.0, // Flag if Volume/Liquidity > 50
            ..
        },
        blacklist: Blacklist {
            tokens: vec!["...".to_string()],
            ..
        },
        queries: vec!["pump".to_string(), "pepe".to_string()],
    }
}
```

## 📊 Data Output
The bot generates `dex_data.jsonl` in the root directory. Each line is a JSON object containing snapshot metrics of the pairs identified by the bot, perfect for further data science or pattern training.

## ⚖️ License
MIT License
