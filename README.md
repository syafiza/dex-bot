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

## 🏁 Launch Guide

### 1. Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) installed.
- A Telegram Bot (create via [@BotFather](https://t.me/botfather)).
- Your Telegram Chat ID (get via [@userinfobot](https://t.me/userinfobot)).

### 2. Configuration
Open `src/main.rs` and update the `TelegramConfig` section with your credentials:

```rust
telegram: TelegramConfig {
    bot_token: "YOUR_BOT_TOKEN".to_string(),
    chat_id: "YOUR_CHAT_ID".to_string(),
    bonkbot_ref: "your_ref".to_string(),
},
```

### 3. Build & Run
```bash
cargo run --release
```

## 🛠 Project Structure (Unified)
- `src/main.rs`: Contains the entire system logic (Scanner, Analysis, Security, Execution).
- `dex_data.jsonl`: Local metrics log for all scanned tokens.

## ⚙️ Security Heuristics
- **Rugcheck.xyz**: Only "Good" status contracts are alerted.
- **Bundle Detection**: Automatic skip if >25% supply is clustered.
- **Fake Volume**: Flags turnover that exceeds liquidity by 50x.

## 📊 Data Output
The bot generates `dex_data.jsonl` in the root directory. Each line is a JSON object containing snapshot metrics of the pairs identified by the bot, perfect for further data science or pattern training.

## ⚖️ License
MIT License
