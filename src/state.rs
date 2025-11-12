//! Settlement Program State Definitions
//! 
//! 定义Settlement Account的数据结构

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Side枚举：Buy或Sell
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

/// 完整的Trade数据（所有19个字段）
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CompleteTrade {
    // === 基础信息 ===
    pub id: String,                    // Trade ID, UUID
    pub market: String,                // "BTC-PERP"
    
    // === 价格和数量（e6格式） ===
    pub price_e6: i64,                 // 105315000000 = 105315 USDC
    pub qty_e6: i64,                   // 1000 = 0.001 BTC
    pub notional_e6: i64,              // price * qty
    
    // === 方向和时间 ===
    pub taker_side: Side,              // Buy/Sell
    pub ts_ms: i64,                    // 成交时间戳（毫秒）
    pub engine_seq: u64,               // 全局序号
    
    // === 订单关联 ===
    pub taker_order_id: String,        // "ord_xxx"
    pub maker_order_id: String,        // "ord_yyy"
    
    // === 账户信息 ===
    pub taker_account_id: String,      // "sol_9ocm..._main"
    pub maker_account_id: String,      // "sol_G23i..._main"
    pub taker_wallet: Pubkey,          // Solana公钥
    pub maker_wallet: Pubkey,          // Solana公钥
    
    // === 杠杆和风险 ===
    pub taker_leverage: u32,           // 20x
    pub maker_leverage: u32,           // 20x
    
    // === 手续费 ===
    pub taker_fee_e6: i64,             // 47391 = 0.047391 USDC
    pub maker_fee_e6: i64,             // 15797 = 0.015797 USDC
    pub fee_rate_taker_bp: u32,        // 45 bp = 0.045%
    pub fee_rate_maker_bp: u32,        // 15 bp = 0.015%
}

/// 账户结算汇总
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SettlementSummary {
    pub account_id: String,            // 账户ID
    pub wallet: Pubkey,                // 钱包地址
    pub margin_change_e6: i64,         // 保证金变化
    pub fee_e6: i64,                   // 手续费
    pub funding_e6: i64,               // 资金费
    pub position_change_e6: i64,       // 持仓变化
}

/// Settlement完整数据
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SettlementData {
    // === 批次信息 ===
    pub batch_id: String,              // Batch UUID
    pub timestamp_ms: i64,             // 批次创建时间
    pub relayer: Pubkey,               // Relayer公钥
    
    // === Trade数据 ===
    pub trades: Vec<CompleteTrade>,    // 完整trade列表
    
    // === 账户汇总 ===
    pub accounts: Vec<SettlementSummary>,
    
    // === 链上信息 ===
    pub block_height: u64,             // 区块高度
    pub tx_signature: [u8; 64],        // 交易签名
    
    // === 验证信息 ===
    pub total_volume_e6: i64,          // 总交易量
    pub total_fees_e6: i64,            // 总手续费
    pub data_hash: [u8; 32],           // 数据SHA256 hash
}

/// Settlement Account（存储在链上）
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SettlementAccount {
    /// Account类型标识符
    pub discriminator: u64,
    
    /// 数据版本
    pub version: u8,
    
    /// PDA bump seed
    pub bump: u8,
    
    /// 预留字段（对齐）
    pub reserved: [u8; 6],
    
    /// Settlement数据
    pub data: SettlementData,
}

impl SettlementAccount {
    /// 账户类型标识符 "SETTLMNT"
    pub const DISCRIMINATOR: u64 = 0x53455454_4c454d54;
    
    /// 当前版本
    pub const VERSION: u8 = 1;
    
    /// Header大小（固定部分）
    pub const HEADER_SIZE: usize = 16;
}

/// Settlement状态枚举
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementStatus {
    Pending,      // 待确认
    Confirmed,    // 已确认
    Finalized,    // 已最终化
    Disputed,     // 争议中
}

