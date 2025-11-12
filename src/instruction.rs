//! Settlement Program Instructions

use borsh::{BorshDeserialize, BorshSerialize};
use crate::state::{SettlementData, SettlementStatus};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum SettlementInstruction {
    /// 记录settlement数据
    /// 
    /// Accounts:
    /// 0. `[writable]` Settlement Account (PDA) - 将被创建
    /// 1. `[signer, writable]` Authority (Relayer) - 支付租金
    /// 2. `[]` System Program
    RecordSettlement {
        /// 完整的settlement数据
        data: SettlementData,
    },
    
    /// 更新settlement状态（预留，可选）
    /// 
    /// Accounts:
    /// 0. `[writable]` Settlement Account
    /// 1. `[signer]` Authority
    UpdateSettlement {
        /// 新的状态
        status: SettlementStatus,
    },
}

