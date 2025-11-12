//! Integration tests for Settlement Program

use borsh::BorshSerialize;
use settlement_program::{
    processor, CompleteTrade, SettlementData, SettlementInstruction, Side,
};
use solana_program::{pubkey::Pubkey, system_program};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn test_record_settlement_success() {
    // 创建测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "settlement_program",
        program_id,
        processor!(processor::process_instruction),
    );
    
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // 创建测试settlement数据
    let batch_id = "79307220-9abf-4f14-a22d-e8b5eebbc40b".to_string();
    
    let trade = CompleteTrade {
        id: "test-trade-1".to_string(),
        market: "BTC-PERP".to_string(),
        price_e6: 105315000000,
        qty_e6: 1000,
        notional_e6: 105315000,
        taker_side: Side::Sell,
        ts_ms: 1762897603000,
        engine_seq: 1,
        taker_order_id: "ord_taker".to_string(),
        maker_order_id: "ord_maker".to_string(),
        taker_account_id: "acc_taker".to_string(),
        maker_account_id: "acc_maker".to_string(),
        taker_wallet: Pubkey::default(),
        maker_wallet: Pubkey::default(),
        taker_leverage: 20,
        maker_leverage: 20,
        taker_fee_e6: 47391,
        maker_fee_e6: 15797,
        fee_rate_taker_bp: 45,
        fee_rate_maker_bp: 15,
    };
    
    let total_volume_e6 = 105315000;
    let total_fees_e6 = 47391 + 15797;
    
    let mut settlement_data = SettlementData {
        batch_id: batch_id.clone(),
        timestamp_ms: 1762897603000,
        relayer: payer.pubkey(),
        trades: vec![trade],
        accounts: vec![],
        block_height: 0,
        tx_signature: [0u8; 64],
        total_volume_e6,
        total_fees_e6,
        data_hash: [0u8; 32],
    };
    
    // 计算hash
    settlement_data.data_hash = settlement_program::utils::calculate_data_hash(&settlement_data);
    
    // 创建instruction
    let instruction_data = SettlementInstruction::RecordSettlement {
        data: settlement_data,
    };
    
    let serialized_instruction = borsh::to_vec(&instruction_data).unwrap();
    
    // 派生PDA
    let (settlement_pda, _bump) = Pubkey::find_program_address(
        &[b"settlement", batch_id.as_bytes()],
        &program_id,
    );
    
    // 创建transaction
    let mut transaction = Transaction::new_with_payer(
        &[solana_program::instruction::Instruction {
            program_id,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(settlement_pda, false),
                solana_program::instruction::AccountMeta::new(payer.pubkey(), true),
                solana_program::instruction::AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: serialized_instruction,
        }],
        Some(&payer.pubkey()),
    );
    
    transaction.sign(&[&payer], recent_blockhash);
    
    // 提交
    let result = banks_client.process_transaction(transaction).await;
    
    // 验证成功
    assert!(result.is_ok(), "Settlement recording failed: {:?}", result.err());
    
    println!("✅ Integration test passed: Settlement recorded successfully!");
}

