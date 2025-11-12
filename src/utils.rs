//! Settlement Program Utility Functions

use solana_program::entrypoint::ProgramResult;

use crate::{
    error::SettlementError,
    state::CompleteTrade,
};

/// 验证trades数据的基本有效性
pub fn validate_settlement_data(trades: &[CompleteTrade]) -> ProgramResult {
    // 1. 验证trades非空
    if trades.is_empty() {
        return Err(SettlementError::EmptyTrades.into());
    }
    
    // 2. 验证每个trade的基本有效性
    for trade in trades {
        // 验证价格和数量为正
        if trade.price_e6 <= 0 || trade.qty_e6 <= 0 {
            return Err(SettlementError::InvalidTrade.into());
        }
        
        // 验证notional计算正确
        let expected_notional = (trade.price_e6 as i128 * trade.qty_e6 as i128 / 1_000_000) as i64;
        if trade.notional_e6 != expected_notional {
            return Err(SettlementError::InvalidTrade.into());
        }
        
        // 验证手续费非负
        if trade.taker_fee_e6 < 0 {
            return Err(SettlementError::InvalidTrade.into());
        }
    }
    
    Ok(())
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
    use crate::state::{CompleteTrade, Side};
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
    
    #[test]
    fn test_validate_correct_trade() {
        let trades = vec![create_test_trade()];
        assert!(validate_settlement_data(&trades).is_ok());
    }
    
    #[test]
    fn test_validate_empty_trades() {
        let trades: Vec<CompleteTrade> = vec![];
        assert!(validate_settlement_data(&trades).is_err());
    }
    
    #[test]
    fn test_validate_invalid_price() {
        let mut trade = create_test_trade();
        trade.price_e6 = 0;
        assert!(validate_settlement_data(&vec![trade]).is_err());
    }
    
    #[test]
    fn test_validate_invalid_notional() {
        let mut trade = create_test_trade();
        trade.notional_e6 = 999999;  // 错误的notional
        assert!(validate_settlement_data(&vec![trade]).is_err());
    }
    
    #[test]
    fn test_validate_batch_id() {
        assert!(validate_batch_id("79307220-9abf-4f14-a22d-e8b5eebbc40b").is_ok());
        assert!(validate_batch_id("invalid-id").is_err());
        assert!(validate_batch_id("").is_err());
    }
}

