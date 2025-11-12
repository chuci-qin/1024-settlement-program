# 完整Trade信息定义

**版本**: 1.0  
**更新时间**: 2025年11月12日

---

## 🎯 设计原则

### 审计完整性

每个trade必须包含**足够的信息**，使得：
1. 可以独立验证成交的正确性
2. 可以追踪订单到成交的完整链路
3. 可以审计手续费计算
4. 可以重建账户状态变化

---

## 📋 完整Trade数据结构

### 核心字段（必需）

```typescript
interface CompleteTrade {
  // === 基础信息 ===
  id: string;                    // Trade唯一ID, UUID v4
  market: string;                // "BTC-PERP", "ETH-PERP"
  
  // === 价格和数量 ===
  price_e6: number;              // 成交价格（e6格式）, 如 105315000000 = 105315 USDC
  qty_e6: number;                // 成交数量（e6格式）, 如 1000 = 0.001 BTC
  notional_e6: number;           // 名义价值（price * qty）, 如 105315000 = 105.315 USDC
  
  // === 方向和时间 ===
  taker_side: "buy" | "sell";    // Taker方向
  ts_ms: number;                 // 成交时间戳（毫秒）, 如 1762897603000
  engine_seq: number;            // 全局单调递增序号, 防重放
  
  // === 订单关联 ===
  taker_order_id: string;        // Taker订单ID, 如 "ord_xxx"
  maker_order_id: string;        // Maker订单ID, 如 "ord_yyy"
  
  // === 账户信息 ===
  taker_account_id: string;      // Taker账户ID, 如 "sol_9ocm..._main"
  maker_account_id: string;      // Maker账户ID, 如 "sol_G23i..._main"
  taker_wallet: string;          // Taker钱包地址（Solana公钥）
  maker_wallet: string;          // Maker钱包地址（Solana公钥）
  
  // === 杠杆和风险 ===
  taker_leverage: number;        // Taker杠杆, 如 20
  maker_leverage: number;        // Maker杠杆, 如 20
  
  // === 手续费 ===
  taker_fee_e6: number;          // Taker手续费（e6）, 如 47391 = 0.047391 USDC
  maker_fee_e6: number;          // Maker手续费（e6）, 如 15797 = 0.015797 USDC (或负数rebate)
  fee_rate_taker: number;        // Taker费率, 如 0.00045 = 0.045%
  fee_rate_maker: number;        // Maker费率, 如 0.00015 = 0.015%
}
```

---

## 🔢 字段详解

### 1. 基础信息

#### id: Trade唯一标识
```
格式: UUID v4
示例: "9811e894-5368-4c1a-8fe3-d149d92279f9"
用途: 全局唯一，防重复，审计追踪
```

#### market: 交易对
```
格式: "{BASE}-{TYPE}"
示例: "BTC-PERP", "ETH-PERP"
用途: 区分不同市场
```

---

### 2. 价格和数量

#### price_e6: 成交价格（e6格式）
```
格式: number (i64)
示例: 105315000000 表示 105315 USDC
公式: actual_price = price_e6 / 1_000_000
用途: 精确记录价格，避免浮点误差
```

#### qty_e6: 成交数量（e6格式）
```
格式: number (i64)
示例: 1000 表示 0.001 BTC
公式: actual_qty = qty_e6 / 1_000_000
用途: 精确记录数量
```

#### notional_e6: 名义价值
```
格式: number (i64)
计算: price * qty
示例: 105315000 = 105.315 USDC
用途: 快速计算交易金额
```

---

### 3. 方向和时间

#### taker_side: Taker方向
```
格式: "buy" | "sell"
示例: "sell"
说明: 
  - Taker sell → 主动卖出
  - Taker buy → 主动买入
  - Maker总是反方向
```

#### ts_ms: 成交时间戳
```
格式: number (i64, 毫秒)
示例: 1762897603000
转换: new Date(1762897603000) → "2025-11-12 03:26:43 UTC"
用途: 时序验证，审计追踪
```

#### engine_seq: 全局序号
```
格式: number (u64)
示例: 7
特性: 单调递增，不重复
用途: 
  - 防重放攻击
  - 验证trade顺序
  - 确保唯一性
```

---

### 4. 订单关联

#### taker_order_id & maker_order_id
```
格式: "ord_{UUID}"
示例: 
  taker: "ord_80cddb72-e3b2-4d5f-8ebb-0256c21b1ed4"
  maker: "ord_e95fb572-a637-4498-a61d-63567099b2af"
用途:
  - 追踪订单到成交
  - 验证撮合逻辑
  - 审计订单执行
```

---

### 5. 账户信息

#### taker_account_id & maker_account_id
```
格式: "{protocol}_{wallet}_{sub_account}"
示例: "sol_9ocm9zv5F2QghKaFSLGSjkVg6f8XZf54nVTjfC2M3dG4_main"
用途:
  - 账户级结算
  - 风险管理
  - 持仓计算
```

#### taker_wallet & maker_wallet
```
格式: Solana公钥（Base58）
示例: "9ocm9zv5F2QghKaFSLGSjkVg6f8XZf54nVTjfC2M3dG4"
用途:
  - 链上验证
  - 提现验证
  - KYC关联
```

---

### 6. 杠杆和风险

#### taker_leverage & maker_leverage
```
格式: number (u32)
示例: 20 表示 20x杠杆
用途:
  - 风险计算
  - 保证金计算
  - 强平价格计算
  
计算:
  Initial Margin = Notional / Leverage
  105.315 / 20 = 5.27 USDC
```

---

### 7. 手续费

#### taker_fee_e6 & maker_fee_e6
```
格式: number (i64, e6)
示例: 
  taker: 47391 = 0.047391 USDC
  maker: 15797 = 0.015797 USDC
  
计算验证:
  Taker: 105.315 * 0.045% = 0.047391 ✅
  Maker: 105.315 * 0.015% = 0.015797 ✅
  
用途:
  - 验证fee计算正确性
  - 审计平台收入
  - 透明度
```

#### fee_rate_taker & fee_rate_maker
```
格式: number (f64)
示例: 
  taker: 0.00045 (0.045%)
  maker: 0.00015 (0.015%)
  
用途:
  - 记录费率（可能动态调整）
  - 验证fee计算
```

---

## 📊 数据大小估算

### JSON格式（完整）

```json
{
  "id": "9811e894-5368-4c1a-8fe3-d149d92279f9",
  "market": "BTC-PERP",
  "price_e6": 105315000000,
  "qty_e6": 1000,
  "notional_e6": 105315000,
  "taker_side": "sell",
  "ts_ms": 1762897603000,
  "engine_seq": 7,
  "taker_order_id": "ord_80cddb72-e3b2-4d5f-8ebb-0256c21b1ed4",
  "maker_order_id": "ord_e95fb572-a637-4498-a61d-63567099b2af",
  "taker_account_id": "sol_9ocm9zv5F2QghKaFSLGSjkVg6f8XZf54nVTjfC2M3dG4_main",
  "maker_account_id": "sol_G23icA8QJiAM2UwENf1112rGFx8bTaYrME3pScMJ4U5t_main",
  "taker_wallet": "9ocm9zv5F2QghKaFSLGSjkVg6f8XZf54nVTjfC2M3dG4",
  "maker_wallet": "G23icA8QJiAM2UwENf1112rGFx8bTaYrME3pScMJ4U5t",
  "taker_leverage": 20,
  "maker_leverage": 20,
  "taker_fee_e6": 47391,
  "maker_fee_e6": 15797,
  "fee_rate_taker": 0.00045,
  "fee_rate_maker": 0.00015
}
```

**大小**: ~800 bytes per trade (格式化JSON)

**压缩后**: ~400 bytes per trade

---

### 精简格式（用于Memo）

```
{id:9811e894,m:BTC-PERP,p:105315,s:0.001,side:sell,ts:1762897603,taker:ord_80cd,maker:ord_e95f}
```

**大小**: ~100 bytes per trade

**2个trades**: ~200 bytes

**加上其他信息**: ~400 bytes total

**结论**: ✅ **Memo有足够空间！**

---

## ✅ 推荐字段优先级

### 必须包含（Memo + IPFS）

1. ⭐⭐⭐⭐⭐ **id** - 唯一标识
2. ⭐⭐⭐⭐⭐ **price** - 成交价格
3. ⭐⭐⭐⭐⭐ **qty** - 成交数量
4. ⭐⭐⭐⭐⭐ **timestamp** - 成交时间
5. ⭐⭐⭐⭐ **market** - 交易对
6. ⭐⭐⭐⭐ **taker_order_id** - Taker订单
7. ⭐⭐⭐⭐ **maker_order_id** - Maker订单

### 应该包含（IPFS）

8. ⭐⭐⭐ **taker_side** - 方向
9. ⭐⭐⭐ **engine_seq** - 全局序号
10. ⭐⭐⭐ **leverage** - 杠杆
11. ⭐⭐⭐ **fee详情** - 手续费

### 可选包含（IPFS）

12. ⭐⭐ **账户快照** - 成交前后状态
13. ⭐⭐ **Position变化** - 持仓变化
14. ⭐ **签名** - 额外验证

---

## 🎁 完整Trade的价值

### 对用户

- ✅ 完全透明
- ✅ 可以验证每笔成交
- ✅ 增强信任

### 对平台

- ✅ 审计追踪完整
- ✅ 争议解决有据
- ✅ 监管合规

### 对生态

- ✅ 去中心化
- ✅ 不可篡改
- ✅ 开放透明

---

**下一步**: [Memo数据扩展设计.md](./Memo数据扩展设计.md)

**时间**: 2025年11月12日 00:58 PST

