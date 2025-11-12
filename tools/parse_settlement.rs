/// Settlement Account数据解析工具
/// 
/// 用法：
/// cargo run --example parse_settlement <settlement_account_address>

use settlement_program::{CompleteTrade, SettlementAccount, SettlementData, Side};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use borsh::BorshDeserialize;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("用法: cargo run --example parse_settlement <settlement_account_address>");
        eprintln!("\n示例:");
        eprintln!("cargo run --example parse_settlement 5vFPPyqdFH3zgt7ArtqE1PFfmuRKaXycdUhmfMkPP9tr");
        std::process::exit(1);
    }
    
    let account_address = &args[1];
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://testnet-rpc.1024chain.com/rpc/".to_string());
    
    println!("🔍 解析Settlement Account数据");
    println!("================================");
    println!("Account: {}", account_address);
    println!("RPC: {}", rpc_url);
    println!();
    
    // 连接RPC
    let rpc_client = RpcClient::new(rpc_url);
    
    // 获取account数据
    let pubkey = Pubkey::from_str(account_address).expect("Invalid public key");
    let account_data = rpc_client.get_account_data(&pubkey).expect("Failed to get account data");
    
    println!("✅ Account数据获取成功");
    println!("   数据大小: {} bytes", account_data.len());
    println!();
    
    // 解析Settlement Account
    let settlement_account = SettlementAccount::try_from_slice(&account_data)
        .expect("Failed to deserialize settlement account");
    
    println!("📦 Settlement Account");
    println!("================================");
    println!("Discriminator: 0x{:016X}", settlement_account.discriminator);
    println!("Version: {}", settlement_account.version);
    println!("Bump: {}", settlement_account.bump);
    println!();
    
    println!("📊 Settlement Data");
    println!("================================");
    println!("Batch ID: {}", settlement_account.data.batch_id);
    println!("Timestamp: {} ({})", 
        settlement_account.data.timestamp_ms,
        chrono::DateTime::from_timestamp_millis(settlement_account.data.timestamp_ms)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "Invalid timestamp".to_string())
    );
    println!("Relayer: {}", settlement_account.data.relayer);
    println!("Total Volume: {} USDC", settlement_account.data.total_volume_e6 as f64 / 1_000_000.0);
    println!("Total Fees: {} USDC", settlement_account.data.total_fees_e6 as f64 / 1_000_000.0);
    println!("Trades Count: {}", settlement_account.data.trades.len());
    println!("Accounts Count: {}", settlement_account.data.accounts.len());
    println!();
    
    // 显示每个trade的完整19字段
    for (i, trade) in settlement_account.data.trades.iter().enumerate() {
        println!("🔹 Trade #{} - 完整19字段", i + 1);
        println!("================================");
        
        println!("【基础信息】");
        println!("  1. ID: {}", trade.id);
        println!("  2. Market: {}", trade.market);
        println!();
        
        println!("【价格和数量】");
        println!("  3. Price: {} USDC (e6: {})", trade.price_e6 as f64 / 1_000_000.0, trade.price_e6);
        println!("  4. Quantity: {} BTC (e6: {})", trade.qty_e6 as f64 / 1_000_000.0, trade.qty_e6);
        println!("  5. Notional: {} USDC (e6: {})", trade.notional_e6 as f64 / 1_000_000.0, trade.notional_e6);
        println!();
        
        println!("【方向和时间】");
        println!("  6. Taker Side: {:?}", trade.taker_side);
        println!("  7. Timestamp: {} ({})",
            trade.ts_ms,
            chrono::DateTime::from_timestamp_millis(trade.ts_ms)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "Invalid".to_string())
        );
        println!("  8. Engine Seq: {}", trade.engine_seq);
        println!();
        
        println!("【订单关联】");
        println!("  9. Taker Order ID: {}", trade.taker_order_id);
        println!(" 10. Maker Order ID: {}", trade.maker_order_id);
        println!();
        
        println!("【账户信息】");
        println!(" 11. Taker Account ID: {}", trade.taker_account_id);
        println!(" 12. Maker Account ID: {}", trade.maker_account_id);
        println!(" 13. Taker Wallet: {}", trade.taker_wallet);
        println!(" 14. Maker Wallet: {}", trade.maker_wallet);
        println!();
        
        println!("【杠杆】");
        println!(" 15. Taker Leverage: {}x", trade.taker_leverage);
        println!(" 16. Maker Leverage: {}x", trade.maker_leverage);
        println!();
        
        println!("【手续费】");
        println!(" 17. Taker Fee: {} USDC (e6: {})", trade.taker_fee_e6 as f64 / 1_000_000.0, trade.taker_fee_e6);
        println!(" 18. Maker Fee: {} USDC (e6: {})", trade.maker_fee_e6 as f64 / 1_000_000.0, trade.maker_fee_e6);
        println!(" 19a. Taker Fee Rate: {}bp ({}%)", trade.fee_rate_taker_bp, trade.fee_rate_taker_bp as f64 / 10000.0);
        println!(" 19b. Maker Fee Rate: {}bp ({}%)", trade.fee_rate_maker_bp, trade.fee_rate_maker_bp as f64 / 10000.0);
        println!();
    }
    
    println!("✅ 解析完成！所有19个字段都已显示！");
}

