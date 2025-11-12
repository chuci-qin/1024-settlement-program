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

/// 用户级Settlement统计账户（每个用户一个）
/// PDA Seeds: [b"user_settlement", user_wallet.as_ref()]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserSettlement {
    /// 账户类型标识符 "USRSETTL" = 0x55535253_4554544c
    pub discriminator: u64,
    
    /// 数据版本
    pub version: u8,
    
    /// PDA bump seed
    pub bump: u8,
    
    /// 预留字段（对齐）
    pub reserved: [u8; 6],
    
    /// 用户钱包地址
    pub wallet: Pubkey,
    
    // === 交易次数统计 ===
    pub total_trades: u64,             // 总交易次数
    pub maker_trades: u64,             // 作为maker的次数
    pub taker_trades: u64,             // 作为taker的次数
    
    // === 交易量统计（USDC, e6格式）===
    pub total_volume_e6: i64,          // 总交易量
    pub maker_volume_e6: i64,          // 作为maker的交易量
    pub taker_volume_e6: i64,          // 作为taker的交易量
    
    // === 手续费统计（USDC, e6格式）===
    pub total_fees_e6: i64,            // 总手续费（净支出，正数=支付）
    pub maker_fees_e6: i64,            // maker手续费（负数=收入）
    pub taker_fees_e6: i64,            // taker手续费（正数=支付）
    
    // === 时间戳 ===
    pub first_trade_ts: i64,           // 首次交易时间（毫秒）
    pub last_trade_ts: i64,            // 最后交易时间（毫秒）
    
    // === 市场统计 ===
    pub btc_perp_trades: u64,          // BTC-PERP交易次数
    pub eth_perp_trades: u64,          // ETH-PERP交易次数
    pub sol_perp_trades: u64,          // SOL-PERP交易次数
    
    // === 预留扩展字段 ===
    pub reserved_stats: [u64; 8],      // 未来扩展用
}

impl UserSettlement {
    /// 账户类型标识符 "USRSETTL"
    pub const DISCRIMINATOR: u64 = 0x55535253_4554544c;
    
    /// 当前版本
    pub const VERSION: u8 = 1;
    
    /// 固定大小（bytes）
    /// 8 + 1 + 1 + 6 + 32 + 8*3 + 8*3 + 8*3 + 8*2 + 8*3 + 8*8 = 232 bytes
    pub const SIZE: usize = 232;
    
    /// 创建新的UserSettlement（初始状态）
    pub fn new(wallet: Pubkey, bump: u8, first_trade_ts: i64) -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
            version: Self::VERSION,
            bump,
            reserved: [0; 6],
            wallet,
            total_trades: 0,
            maker_trades: 0,
            taker_trades: 0,
            total_volume_e6: 0,
            maker_volume_e6: 0,
            taker_volume_e6: 0,
            total_fees_e6: 0,
            maker_fees_e6: 0,
            taker_fees_e6: 0,
            first_trade_ts,
            last_trade_ts: first_trade_ts,
            btc_perp_trades: 0,
            eth_perp_trades: 0,
            sol_perp_trades: 0,
            reserved_stats: [0; 8],
        }
    }
    
    /// 更新统计（作为taker）
    pub fn update_as_taker(&mut self, trade: &CompleteTrade) {
        self.total_trades += 1;
        self.taker_trades += 1;
        
        let volume = (trade.price_e6 as i128 * trade.qty_e6 as i128 / 1_000_000) as i64;
        self.total_volume_e6 += volume;
        self.taker_volume_e6 += volume;
        
        self.total_fees_e6 += trade.taker_fee_e6;
        self.taker_fees_e6 += trade.taker_fee_e6;
        
        self.last_trade_ts = trade.ts_ms;
        
        // 更新市场统计
        self.update_market_stats(&trade.market);
    }
    
    /// 更新统计（作为maker）
    pub fn update_as_maker(&mut self, trade: &CompleteTrade) {
        self.total_trades += 1;
        self.maker_trades += 1;
        
        let volume = (trade.price_e6 as i128 * trade.qty_e6 as i128 / 1_000_000) as i64;
        self.total_volume_e6 += volume;
        self.maker_volume_e6 += volume;
        
        // Maker手续费通常是负数（收入）
        self.total_fees_e6 += trade.maker_fee_e6;
        self.maker_fees_e6 += trade.maker_fee_e6;
        
        self.last_trade_ts = trade.ts_ms;
        
        // 更新市场统计
        self.update_market_stats(&trade.market);
    }
    
    /// 更新市场统计
    fn update_market_stats(&mut self, market: &str) {
        match market {
            "BTC-PERP" => self.btc_perp_trades += 1,
            "ETH-PERP" => self.eth_perp_trades += 1,
            "SOL-PERP" => self.sol_perp_trades += 1,
            _ => {}, // 其他市场暂不统计
        }
    }
}

