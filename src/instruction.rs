//! Settlement Program Instructions

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use crate::state::CompleteTrade;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum SettlementInstruction {
    /// 初始化用户的Settlement账户（首次交易时）
    /// 
    /// Accounts:
    /// 0. `[writable]` UserSettlement PDA - 将被创建
    /// 1. `[signer, writable]` Authority (Relayer) - 支付租金
    /// 2. `[]` System Program
    InitializeUser {
        /// 用户钱包地址
        wallet: Pubkey,
    },
    
    /// 记录Settlement并更新用户统计
    /// 
    /// Accounts:
    /// 0..N. `[writable]` UserSettlement PDAs (所有涉及的用户)
    /// N+1. `[signer]` Authority (Relayer)
    /// 
    /// 注意：accounts顺序必须是先taker，后maker，按trades顺序排列
    RecordSettlement {
        /// Batch ID（用于日志）
        batch_id: String,
        /// 完整的trade列表
        trades: Vec<CompleteTrade>,
    },
}

