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

### 🏁 Launch Guide (Python - Recommended for Windows)
Due to toolchain requirements for the high-performance Rust core, we recommend using the **Python version** for immediate Paper Trading on Windows.

1.  **Install dependencies**:
    ```bash
    pip install requests python-dotenv colorama
    ```
2.  **Configure**: Create a `.env` file with:
    ```env
    TELEGRAM_BOT_TOKEN=your_token
    TELEGRAM_CHAT_ID=your_id
    ```
3.  **Run**:
    ```bash
    python bot.py
    ```

### 🦀 Rust Implementation (Advanced)
The high-performance Rust core is available for Linux/WSL environments or systems with a full C++ Build Toolchain.
```bash
cargo run --release
```

**Backtest Mode (Replay historical data):**
```bash
cargo run --release -- --backtest
```

## 🛠 Project Structure (Unified)
- `src/main.rs`: Scanner, Analysis, Security Engine, Paper Trading, and Backtesting Logic.
- `dex_data.jsonl`: Local metrics log (used for Backtesting).

## ⚙️ Key Engine Features
- **PostgreSQL Support**: Structured logging of all scans and trades.
- **Historical Backtesting**: Replay `dex_data.jsonl` to verify filter performance.
- **Paper Trading**: Automatically simulates buys on "Good" signals.
- **Rugcheck.xyz**: Only "Good" status contracts are considered.
- **Bundle Detection**: Automatic skip if >25% supply is clustered.
- **Fake Volume**: Flags turnover that exceeds liquidity by 50x.

## 📊 Data Output
The bot generates `dex_data.jsonl` in the root directory. Each line is a JSON object containing snapshot metrics of the pairs identified by the bot, perfect for further data science or pattern training.

## ⚖️ License
MIT License
