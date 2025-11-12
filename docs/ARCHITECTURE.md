# 数据结构设计

**版本**: 1.0  
**更新时间**: 2025年11月12日

---

## 📋 核心数据结构

### 1. CompleteTrade（完整Trade）

```rust
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CompleteTrade {
    // === 基础信息 ===
    pub id: String,                    // Trade ID, 36字符UUID
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
    pub taker_wallet: [u8; 32],        // Solana公钥（二进制）
    pub maker_wallet: [u8; 32],        // Solana公钥（二进制）
    
    // === 杠杆和风险 ===
    pub taker_leverage: u32,           // 20x
    pub maker_leverage: u32,           // 20x
    
    // === 手续费 ===
    pub taker_fee_e6: i64,             // 47391
    pub maker_fee_e6: i64,             // 15797
    pub fee_rate_taker_bp: u32,        // 45 bp (0.045%)
    pub fee_rate_maker_bp: u32,        // 15 bp (0.015%)
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub enum Side {
    Buy,
    Sell,
}
```

**大小估算**:
- Strings: ~150 bytes (id + market + order_ids + account_ids)
- Numbers: ~100 bytes (i64, u64, u32)
- Pubkeys: 64 bytes (2个钱包)
- **总计**: ~314 bytes per trade

---

### 2. SettlementSummary（账户汇总）

```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SettlementSummary {
    pub account_id: String,            // 账户ID
    pub wallet: [u8; 32],              // 钱包地址
    pub margin_change_e6: i64,         // 保证金变化
    pub fee_e6: i64,                   // 手续费
    pub funding_e6: i64,               // 资金费（通常为0）
    pub position_change_e6: i64,       // 持仓变化
}
```

**大小**: ~120 bytes per account

---

### 3. SettlementData（完整Settlement）

```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SettlementData {
    // === 批次信息 ===
    pub batch_id: String,              // Batch UUID
    pub timestamp_ms: i64,             // 批次创建时间
    pub relayer: [u8; 32],             // Relayer公钥
    
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
```

**大小估算**（2个trades, 2个accounts）:
- 固定字段: ~200 bytes
- Trades: 314 * 2 = 628 bytes
- Accounts: 120 * 2 = 240 bytes
- **总计**: ~1068 bytes ✅

**100个trades**:
- 固定: ~200 bytes
- Trades: 314 * 100 = 31,400 bytes
- Accounts: 120 * 100 = 12,000 bytes
- **总计**: ~43KB

---

## 🔧 Program Instruction

### RecordSettlement指令

```rust
#[derive(BorshSerialize, BorshDeserialize)]
pub enum SettlementInstruction {
    /// 记录settlement数据
    /// 
    /// Accounts:
    /// 0. `[writable]` Settlement Account - 存储数据
    /// 1. `[signer]` Authority - Relayer权限
    /// 2. `[]` System Program
    RecordSettlement {
        data: SettlementData,
    },
    
    /// 查询settlement数据
    /// 
    /// Accounts:
    /// 0. `[]` Settlement Account
    QuerySettlement {
        batch_id: String,
    },
}
```

---

## 💾 Account布局

### Settlement Account

**Account地址生成**（PDA）:
```rust
// 使用batch_id作为seed
let (settlement_account, bump) = Pubkey::find_program_address(
    &[
        b"settlement",
        batch_id.as_bytes(),
    ],
    &program_id,
);
```

**Account数据**:
```
[Account Header (固定)]
├─ discriminator: u64 (8 bytes) - 账户类型标识
├─ version: u8 (1 byte) - 数据版本
├─ bump: u8 (1 byte) - PDA bump seed
└─ reserved: [u8; 6] (6 bytes) - 预留

[Settlement Data (可变)]
└─ data: SettlementData (Borsh序列化)
    ├─ batch_id: String
    ├─ trades: Vec<CompleteTrade>
    ├─ accounts: Vec<SettlementSummary>
    └─ ... 其他字段
```

**总大小**: 16 bytes (header) + settlement_data_size

---

## 🎯 数据容量规划

### Account租金计算

**Solana租金公式**:
```
Rent = (Account Size) * Lamports_per_byte_year * Years
```

**1024Chain N1024租金**（假设与Solana类似）:
- 1KB数据 ≈ 0.001 N1024 per year
- 10KB数据 ≈ 0.01 N1024 per year
- 50KB数据 ≈ 0.05 N1024 per year

**成本极低！**

---

### 不同批次大小

| Trades数量 | Account大小 | 租金/年 | 适用场景 |
|-----------|------------|---------|----------|
| 1-10 | ~5KB | 0.005 N1024 | 低频交易 |
| 10-50 | ~20KB | 0.02 N1024 | 中频交易 |
| 50-100 | ~40KB | 0.04 N1024 | 高频交易 |
| 100+ | 分片存储 | 按片计算 | 极高频 |

**结论**: ✅ **成本完全可接受！**

---

## 🔐 安全设计

### 权限控制

```rust
// 只有授权的Relayer可以记录settlement
pub fn process_record_settlement(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: SettlementData,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let settlement_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;
    
    // 验证authority是预设的Relayer
    if authority.key != &AUTHORIZED_RELAYER {
        return Err(ProgramError::InvalidAuthority);
    }
    
    // 验证authority签名
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // ... 存储数据
}
```

---

### 数据验证

```rust
// 验证settlement数据的完整性
fn validate_settlement_data(data: &SettlementData) -> ProgramResult {
    // 1. 验证trades非空
    if data.trades.is_empty() {
        return Err(SettlementError::EmptyTrades.into());
    }
    
    // 2. 验证总量计算正确
    let calculated_volume: i64 = data.trades.iter()
        .map(|t| (t.price_e6 as i128 * t.qty_e6 as i128 / 1_000_000) as i64)
        .sum();
    
    if calculated_volume != data.total_volume_e6 {
        return Err(SettlementError::InvalidTotalVolume.into());
    }
    
    // 3. 验证总手续费
    let calculated_fees: i64 = data.trades.iter()
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
```

---

## 🎁 核心特性

### 1. 完全去中心化
- ✅ 所有数据在1024Chain
- ✅ 不依赖IPFS
- ✅ 不依赖中心化数据库

### 2. 完整审计
- ✅ 所有trade详情
- ✅ 费用计算可验证
- ✅ 时序可验证

### 3. 高性能
- ✅ Borsh序列化（快速）
- ✅ 批量处理
- ✅ Account分片（大批次）

### 4. 可扩展
- ✅ Program可升级
- ✅ 数据格式可扩展
- ✅ 未来可添加新功能

---

**下一步**: [03-Program架构.md](./03-Program架构.md)

**时间**: 2025年11月12日 01:03 PST

