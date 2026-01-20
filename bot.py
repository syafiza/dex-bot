import os
import time
import json
import requests
from datetime import datetime
from dotenv import load_dotenv
from colorama import Fore, Style, init

# Initialize Colorama
init(autoreset=True)
load_dotenv()

# =============================================================================
# CONFIGURATION
# =============================================================================

CONFIG = {
    "queries": ["pump", "moon", "solana"],
    "filters": {
        "min_liquidity_usd": 1000.0,
        "min_volume_h24_usd": 5000.0,
        "min_mcap_usd": 10000.0,
        "max_vlr": 50.0,
        "max_bundled_supply_percent": 25.0,
    },
    "blacklist": ["0x0000000000000000000000000000000000000000"],
    "telegram": {
        "bot_token": os.getenv("TELEGRAM_BOT_TOKEN", ""),
        "chat_id": os.getenv("TELEGRAM_CHAT_ID", ""),
        "bonkbot_ref": "ref_code",
    },
    "paper_trading": {
        "enabled": True,
        "buy_amount_sol": 0.1,
        "take_profit_percent": 50.0,
        "stop_loss_percent": 25.0,
    },
    "scan_interval": 60,  # seconds
}

# =============================================================================
# DATA STORAGE
# =============================================================================

ACTIVE_TRADES = []

def log_scan(pair, pattern):
    record = {
        "ts": int(time.time()),
        "addr": pair.get("pairAddress"),
        "sym": pair.get("baseToken", {}).get("symbol"),
        "pattern": pattern
    }
    with open("dex_data.jsonl", "a") as f:
        f.write(json.dumps(record) + "\n")

# =============================================================================
# TELEGRAM NOTIFICATIONS
# =============================================================================

def send_telegram_msg(message):
    token = CONFIG["telegram"]["bot_token"]
    chat_id = CONFIG["telegram"]["chat_id"]
    if not token or not chat_id:
        return
    
    url = f"https://api.telegram.org/bot{token}/sendMessage"
    payload = {
        "chat_id": chat_id,
        "text": message,
        "parse_mode": "MarkdownV2"
    }
    try:
        requests.post(url, json=payload, timeout=10)
    except Exception as e:
        print(f"Error sending TG msg: {e}")

# =============================================================================
# ANALYSIS ENGINE
# =============================================================================

def analyze_pair(pair, rug_report=None):
    addr = pair.get("pairAddress")
    if addr in CONFIG["blacklist"]:
        return "Blacklisted"

    # Rugcheck Verification
    if rug_report:
        if rug_report.get("status") != "good":
            return "RugcheckRisk"
        
        meta = rug_report.get("file_meta")
        if meta and meta.get("bundle_ratio"):
            if meta["bundle_ratio"] * 100.0 > CONFIG["filters"]["max_bundled_supply_percent"]:
                return "BundledSupply"

    # Heuristics
    liq_usd = pair.get("liquidity", {}).get("usd", 0)
    vol_h24 = pair.get("volume", {}).get("h24", 0)
    mcap = pair.get("marketCap", 0)

    if liq_usd > 0:
        vlr = vol_h24 / liq_usd
        if vlr > CONFIG["filters"]["max_vlr"]:
            return "FakeVolume"
        
    if liq_usd < CONFIG["filters"]["min_liquidity_usd"]:
        return "LowLiquidity"
    
    if vol_h24 < CONFIG["filters"]["min_volume_h24_usd"]:
        return "LowVolume"
    
    if mcap and mcap < CONFIG["filters"]["min_mcap_usd"]:
        return "LowMcap"

    # Growth check (M5)
    m5_change = pair.get("priceChange", {}).get("m5", 0)
    if m5_change:
        if 5.0 < m5_change < 50.0:
            return "GoodCandidate"

    return "Unknown"

# =============================================================================
# PAPER TRADING UTILS
# =============================================================================

def monitor_paper_trades():
    global ACTIVE_TRADES
    to_remove = []
    
    for i, trade in enumerate(ACTIVE_TRADES):
        try:
            url = f"https://api.dexscreener.com/latest/dex/tokens/{trade['address']}"
            resp = requests.get(url, timeout=10).json()
            pairs = resp.get("pairs", [])
            if not pairs:
                continue
            
            pair = pairs[0]
            current_price = float(pair.get("priceUsd", 0))
            if current_price == 0:
                continue
                
            pnl = ((current_price - trade["entry_price"]) / trade["entry_price"]) * 100.0
            print(f"{Fore.CYAN}📊 [PAPER TRADE] {trade['symbol']} PnL: {pnl:.2f}%")
            
            if pnl >= CONFIG["paper_trading"]["take_profit_percent"] or pnl <= -CONFIG["paper_trading"]["stop_loss_percent"]:
                status = "PROFIT" if pnl > 0 else "LOSS"
                print(f"{Fore.YELLOW}📉 [PAPER TRADE] EXIT {status}: {trade['symbol']} at {pnl:.2f}%")
                
                msg = f"🔔 *PAPER TRADE CLOSED*\n\nToken: {trade['symbol']}\nResult: {pnl:.2f}%\nExit Price: ${current_price:.8f}"
                send_telegram_msg(msg.replace(".", "\\.")) # Simple escaping for MarkdownV2
                to_remove.append(i)
                
        except Exception as e:
            print(f"Error monitoring trade {trade['symbol']}: {e}")
            
    for idx in sorted(to_remove, reverse=True):
        ACTIVE_TRADES.pop(idx)

# =============================================================================
# MAIN LOOP
# =============================================================================

def main():
    print(f"{Fore.GREEN}🚀 Starting Consolidated DexBot (Python Edition)...")
    print(f"{Fore.BLUE}Shadow Mode Reactive: Monitoring signals and paper trading...")

    while True:
        for query in CONFIG["queries"]:
            print(f"{Fore.WHITE}🔍 Scanning: {query}...")
            
            try:
                url = f"https://api.dexscreener.com/latest/dex/search?q={query}"
                data = requests.get(url, timeout=10).json()
                pairs = data.get("pairs", [])
                
                for pair in pairs:
                    addr = pair.get("baseToken", {}).get("address")
                    if not addr: continue
                    
                    # Security Check: Rugcheck
                    try:
                        rug_url = f"https://api.rugcheck.xyz/v1/tokens/{addr}/report"
                        rug_report = requests.get(rug_url, timeout=5).json()
                    except:
                        rug_report = None
                        
                    pattern = analyze_pair(pair, rug_report)
                    log_scan(pair, pattern)
                    
                    if pattern == "GoodCandidate":
                        sym = pair.get("baseToken", {}).get("symbol")
                        price = float(pair.get("priceUsd", 0))
                        
                        # Already in trade?
                        if any(t["address"] == addr for t in ACTIVE_TRADES):
                            continue
                            
                        print(f"{Fore.GREEN}✅ SIGNAL: {sym} found.")
                        
                        # Process Paper Trade
                        if CONFIG["paper_trading"]["enabled"]:
                            ACTIVE_TRADES.append({
                                "address": addr,
                                "symbol": sym,
                                "entry_price": price,
                                "entry_time": time.time()
                            })
                            print(f"{Fore.CYAN}📈 [PAPER TRADE] ENTER: {sym} at ${price:.8f}")
                        
                        # Notify Telegram
                        ref = CONFIG["telegram"]["bonkbot_ref"]
                        bonk_link = f"https://t.me/bonkbot_bot?start={ref}_{addr}"
                        mcap = pair.get("marketCap", "N/A")
                        liq = pair.get("liquidity", {}).get("usd", "N/A")
                        vol = pair.get("volume", {}).get("h24", 0)
                        
                        msg = (f"💎 *GOOD SIGNAL: {sym}*\n\n"
                               f"💰 Mcap: ${mcap}\n"
                               f"💧 Liq: ${liq}\n"
                               f"📈 Vol: ${vol:.2f}\n\n"
                               f"[🚀 OPEN IN BONKBOT]({bonk_link})")
                        send_telegram_msg(msg.replace(".", "\\.").replace("-", "\\-"))
                        
                time.sleep(5) # Small gap between queries
                
            except Exception as e:
                print(f"Error in scan loop: {e}")

        # Monitor current paper trades after each scan cycle
        if ACTIVE_TRADES:
            monitor_paper_trades()
            
        print(f"Cycle complete. Passive monitoring for {CONFIG['scan_interval']}s...")
        time.sleep(CONFIG["scan_interval"])

if __name__ == "__main__":
    main()
