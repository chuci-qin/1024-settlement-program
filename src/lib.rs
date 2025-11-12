//! 1024EX Settlement Program
//! 
//! 自定义Solana Program，用于在1024Chain上存储完整的settlement数据。
//! 
//! 核心特性：
//! - 无大小限制（10KB+）
//! - 完整trade数据（19个字段）
//! - Borsh序列化
//! - PDA存储
//! - 数据验证和hash

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;

// Settlement Program ID (已部署到1024Chain Testnet)
// 部署日期: 2025-11-12
// Transaction: 3Zh7cVFJuhip56kU6y7z6HprjQvbM8iGJYkJGtAqUxjiw4ukUtvx2Aoeuf96GgnfuGPJF96pVFRvq76mFuRkvcmh
solana_program::declare_id!("D1VtiVDF1iVojfVUpuyxkjixnYdr2cdwrakQgqJ5QJ7w");

// Program入口点
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processor::process_instruction(program_id, accounts, instruction_data)
}

// 导出公共类型，方便客户端使用
pub use error::SettlementError;
pub use instruction::SettlementInstruction;
pub use state::{
    CompleteTrade, SettlementAccount, SettlementData, SettlementStatus, SettlementSummary, Side,
};

