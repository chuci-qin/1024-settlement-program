/// Basic Settlement Example
/// 
/// This example demonstrates how to create and submit a basic settlement

use settlement_program::{
    CompleteTrade, SettlementData, SettlementInstruction, SettlementSummary, Side,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;

fn main() {
    println!("=== Basic Settlement Example ===\n");

    // 1. Create a sample trade
    let trade = CompleteTrade {
        id: "trade_example_001".to_string(),
        market: "BTC-PERP".to_string(),
        
        // Price: 105,315 USDC (e6 format)
        price_e6: 105315000000,
        
        // Quantity: 0.001 BTC (e6 format)
        qty_e6: 1000,
        
        // Notional: price * qty / 1_000_000
        notional_e6: 105315000,
        
        taker_side: Side::Sell,
        ts_ms: chrono::Utc::now().timestamp_millis(),
        engine_seq: 1,
        
        taker_order_id: "ord_taker_001".to_string(),
        maker_order_id: "ord_maker_001".to_string(),
        
        taker_account_id: "account_taker".to_string(),
        maker_account_id: "account_maker".to_string(),
        taker_wallet: Pubkey::default(),
        maker_wallet: Pubkey::default(),
        
        taker_leverage: 20,
        maker_leverage: 20,
        
        // Taker fee: 0.045% = 47,391
        taker_fee_e6: 47391,
        
        // Maker fee: 0.015% = 15,797
        maker_fee_e6: 15797,
        
        fee_rate_taker_bp: 45,  // 0.045%
        fee_rate_maker_bp: 15,  // 0.015%
    };

    println!("Trade created:");
    println!("  ID: {}", trade.id);
    println!("  Market: {}", trade.market);
    println!("  Price: {} USDC", trade.price_e6 as f64 / 1_000_000.0);
    println!("  Quantity: {} BTC", trade.qty_e6 as f64 / 1_000_000.0);
    println!("  Notional: {} USDC", trade.notional_e6 as f64 / 1_000_000.0);
    println!();

    // 2. Create settlement data
    let batch_id = uuid::Uuid::new_v4().to_string();
    let total_volume_e6 = trade.notional_e6;
    let total_fees_e6 = trade.taker_fee_e6 + trade.maker_fee_e6;

    let mut settlement_data = SettlementData {
        batch_id: batch_id.clone(),
        timestamp_ms: chrono::Utc::now().timestamp_millis(),
        relayer: Pubkey::default(),
        trades: vec![trade],
        accounts: vec![],
        block_height: 0,
        tx_signature: [0u8; 64],
        total_volume_e6,
        total_fees_e6,
        data_hash: [0u8; 32],
    };

    // Calculate data hash
    settlement_data.data_hash = calculate_hash(&settlement_data);

    println!("Settlement data created:");
    println!("  Batch ID: {}", settlement_data.batch_id);
    println!("  Total volume: {} USDC", total_volume_e6 as f64 / 1_000_000.0);
    println!("  Total fees: {} USDC", total_fees_e6 as f64 / 1_000_000.0);
    println!("  Trades: {}", settlement_data.trades.len());
    println!();

    // 3. Create instruction
    let instruction_data = SettlementInstruction::RecordSettlement {
        data: settlement_data,
    };

    println!("Instruction created: RecordSettlement");
    println!();

    // 4. Derive PDA
    let program_id = Pubkey::from_str("Sett1ementProg11111111111111111111111111111")
        .unwrap();
    
    let (settlement_pda, bump) = Pubkey::find_program_address(
        &[b"settlement", batch_id.as_bytes()],
        &program_id,
    );

    println!("PDA derived:");
    println!("  Address: {}", settlement_pda);
    println!("  Bump: {}", bump);
    println!();

    println!("✅ Settlement prepared successfully!");
    println!("\nNext steps:");
    println!("  1. Serialize instruction_data with borsh");
    println!("  2. Create Solana transaction");
    println!("  3. Sign with authority");
    println!("  4. Send to 1024Chain");
}

// Helper function to calculate hash
fn calculate_hash(data: &SettlementData) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    
    let mut hasher = Sha256::new();
    hasher.update(data.batch_id.as_bytes());
    hasher.update(&data.timestamp_ms.to_le_bytes());
    
    for trade in &data.trades {
        hasher.update(trade.id.as_bytes());
        hasher.update(&trade.price_e6.to_le_bytes());
        hasher.update(&trade.qty_e6.to_le_bytes());
    }
    
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

