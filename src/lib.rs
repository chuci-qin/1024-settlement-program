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

// 声明Program ID（部署后会更新为实际ID）
// 临时使用占位符
solana_program::declare_id!("Sett1ementProg11111111111111111111111111111");

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

