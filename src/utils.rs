//! Settlement Program Utility Functions

use sha2::{Digest, Sha256};
use solana_program::entrypoint::ProgramResult;

use crate::{
    error::SettlementError,
    state::SettlementData,
};


/// 验证settlement数据的完整性
pub fn validate_settlement_data(data: &SettlementData) -> ProgramResult {
    // 1. 验证trades非空
    if data.trades.is_empty() {
        return Err(SettlementError::EmptyTrades.into());
    }
    
    // 2. 验证总交易量
    let calculated_volume: i64 = data
        .trades
        .iter()
        .map(|t| {
            // notional = price * qty / 1_000_000
            (t.price_e6 as i128 * t.qty_e6 as i128 / 1_000_000) as i64
        })
        .sum();
    
    if calculated_volume != data.total_volume_e6 {
        return Err(SettlementError::InvalidTotalVolume.into());
    }
    
    // 3. 验证总手续费
    let calculated_fees: i64 = data
        .trades
        .iter()
        .map(|t| t.taker_fee_e6 + t.maker_fee_e6)
        .sum();
    
    if calculated_fees != data.total_fees_e6 {
        return Err(SettlementError::InvalidTotalFees.into());
    }
    
    // 4. 验证数据hash
    let computed_hash = calculate_data_hash(data);
    if computed_hash != data.data_hash {
        return Err(SettlementError::InvalidDataHash.into());
    }
    
    Ok(())
}

/// 计算settlement数据的SHA256 hash
pub fn calculate_data_hash(data: &SettlementData) -> [u8; 32] {
    let mut hasher = Sha256::new();
    
    // Hash batch_id
    hasher.update(data.batch_id.as_bytes());
    
    // Hash timestamp
    hasher.update(&data.timestamp_ms.to_le_bytes());
    
    // Hash relayer
    hasher.update(data.relayer.as_ref());
    
    // Hash每个trade的关键字段
    for trade in &data.trades {
        hasher.update(trade.id.as_bytes());
        hasher.update(&trade.price_e6.to_le_bytes());
        hasher.update(&trade.qty_e6.to_le_bytes());
        hasher.update(&trade.ts_ms.to_le_bytes());
        hasher.update(&(trade.engine_seq as u64).to_le_bytes());
    }
    
    // Hash accounts
    for account in &data.accounts {
        hasher.update(account.account_id.as_bytes());
        hasher.update(&account.margin_change_e6.to_le_bytes());
        hasher.update(&account.fee_e6.to_le_bytes());
    }
    
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// 验证batch_id格式（UUID）
pub fn validate_batch_id(batch_id: &str) -> ProgramResult {
    // 简单验证：UUID格式为 xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    if batch_id.len() != 36 {
        return Err(SettlementError::InvalidBatchId.into());
    }
    
    if !batch_id.chars().enumerate().all(|(i, c)| {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            c == '-'
        } else {
            c.is_ascii_hexdigit()
        }
    }) {
        return Err(SettlementError::InvalidBatchId.into());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{CompleteTrade, SettlementSummary, Side};
    use solana_program::pubkey::Pubkey;
    
    fn create_test_trade() -> CompleteTrade {
        CompleteTrade {
            id: "9811e894-5368-4c1a-8fe3-d149d92279f9".to_string(),
            market: "BTC-PERP".to_string(),
            price_e6: 105315000000,
            qty_e6: 1000,
            notional_e6: 105315000,
            taker_side: Side::Sell,
            ts_ms: 1762897603000,
            engine_seq: 7,
            taker_order_id: "ord_80cddb72-e3b2-4d5f-8ebb-0256c21b1ed4".to_string(),
            maker_order_id: "ord_e95fb572-a637-4498-a61d-63567099b2af".to_string(),
            taker_account_id: "sol_9ocm9zv5F2QghKaFSLGSjkVg6f8XZf54nVTjfC2M3dG4_main".to_string(),
            maker_account_id: "sol_G23icA8QJiAM2UwENf1112rGFx8bTaYrME3pScMJ4U5t_main".to_string(),
            taker_wallet: Pubkey::default(),
            maker_wallet: Pubkey::default(),
            taker_leverage: 20,
            maker_leverage: 20,
            taker_fee_e6: 47391,
            maker_fee_e6: 15797,
            fee_rate_taker_bp: 45,
            fee_rate_maker_bp: 15,
        }
    }
    
    fn create_test_settlement_data(trade_count: usize) -> SettlementData {
        let trades: Vec<CompleteTrade> = (0..trade_count)
            .map(|i| {
                let mut trade = create_test_trade();
                trade.id = format!("trade-{}", i);
                trade.engine_seq = i as u64;
                trade
            })
            .collect();
        
        let total_volume_e6 = trades
            .iter()
            .map(|t| (t.price_e6 as i128 * t.qty_e6 as i128 / 1_000_000) as i64)
            .sum();
        
        let total_fees_e6 = trades
            .iter()
            .map(|t| t.taker_fee_e6 + t.maker_fee_e6)
            .sum();
        
        let mut data = SettlementData {
            batch_id: "79307220-9abf-4f14-a22d-e8b5eebbc40b".to_string(),
            timestamp_ms: 1762897603000,
            relayer: Pubkey::default(),
            trades,
            accounts: vec![],
            block_height: 0,
            tx_signature: [0u8; 64],
            total_volume_e6,
            total_fees_e6,
            data_hash: [0u8; 32],
        };
        
        data.data_hash = calculate_data_hash(&data);
        data
    }
    
    #[test]
    fn test_hash_consistency() {
        let data = create_test_settlement_data(2);
        let hash1 = calculate_data_hash(&data);
        let hash2 = calculate_data_hash(&data);
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_hash_uniqueness() {
        let data1 = create_test_settlement_data(2);
        let mut data2 = data1.clone();
        data2.trades[0].price_e6 += 1;
        
        let hash1 = calculate_data_hash(&data1);
        let hash2 = calculate_data_hash(&data2);
        assert_ne!(hash1, hash2);
    }
    
    #[test]
    fn test_validate_correct_data() {
        let data = create_test_settlement_data(2);
        assert!(validate_settlement_data(&data).is_ok());
    }
    
    #[test]
    fn test_validate_empty_trades() {
        let data = create_test_settlement_data(0);
        assert!(validate_settlement_data(&data).is_err());
    }
    
    #[test]
    fn test_validate_invalid_volume() {
        let mut data = create_test_settlement_data(2);
        data.total_volume_e6 = 999999;
        assert!(validate_settlement_data(&data).is_err());
    }
    
    #[test]
    fn test_validate_batch_id() {
        assert!(validate_batch_id("79307220-9abf-4f14-a22d-e8b5eebbc40b").is_ok());
        assert!(validate_batch_id("invalid-id").is_err());
        assert!(validate_batch_id("").is_err());
    }
}

