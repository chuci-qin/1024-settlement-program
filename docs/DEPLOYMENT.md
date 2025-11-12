# Settlement Program - 快速部署指南

**适用人员**: 运维、开发  
**预计时间**: 1小时  
**前置条件**: Rust工具链已更新

---

## 📋 准备工作

### 1. 更新Rust工具链（如果需要）

```bash
# 检查当前版本
rustc --version
cargo --version

# 更新到最新版本
rustup update

# 或使用nightly版本（如果stable版本不支持）
rustup default nightly
rustup update
```

### 2. 检查Solana CLI

```bash
# 检查版本
solana --version

# 配置网络
solana config set --url https://testnet-rpc.1024chain.com/rpc/

# 配置keypair
solana config set --keypair /Users/chuciqin/Desktop/project1024/1024codebase/1024-core/settlement-authority-fixed.json

# 检查余额
solana balance

# 如果余额不足，需要申请测试代币
# （联系1024Chain团队或使用水龙头）
```

---

## 🔨 步骤1: 编译BPF

```bash
# 进入program目录
cd /Users/chuciqin/Desktop/project1024/1024codebase/1024-core/programs/settlement-program

# 编译BPF（或SBF）
cargo build-sbf

# 或者如果工具链较旧
cargo build-bpf

# 成功输出：
# To deploy this program:
#   solana program deploy /path/to/settlement_program.so
```

**预期结果**:
- 生成 `target/deploy/settlement_program.so`
- 文件大小约 50-200KB

**如果失败**:
- 检查Rust版本是否最新
- 检查错误信息，可能需要更新依赖
- 尝试 `cargo clean` 后重新编译

---

## 🚀 步骤2: 部署Program

```bash
# 确保在settlement-program目录
cd /Users/chuciqin/Desktop/project1024/1024codebase/1024-core/programs/settlement-program

# 部署
solana program deploy target/deploy/settlement_program.so

# 成功输出：
# Program Id: <PROGRAM_ID_HERE>
```

**记录Program ID**:

```bash
# 示例输出：
Program Id: 7xJ8... （实际是44个字符的Base58）

# 复制这个ID，后面会用到
```

**检查部署**:

```bash
# 查询program信息
solana program show <PROGRAM_ID>

# 应该显示：
# Program Id: <PROGRAM_ID>
# Owner: BPFLoaderUpgradeab1e11111111111111111111111
# Data Length: XXXXX bytes
```

---

## ⚙️ 步骤3: 更新代码中的Program ID

### 3.1 更新Program源代码

编辑 `programs/settlement-program/src/lib.rs`:

```rust
// 第24行，替换占位符为实际ID
solana_program::declare_id!("<实际的PROGRAM_ID>");
```

### 3.2 重新编译（验证）

```bash
# 编译库版本（不是BPF）
cargo build --lib

# 应该编译成功，没有错误
```

**注意**: 这一步只是更新代码，不需要重新部署

---

## 🔧 步骤4: 配置环境变量

### 4.1 创建配置文件

```bash
# 进入1024-core目录
cd /Users/chuciqin/Desktop/project1024/1024codebase/1024-core

# 复制示例配置
cp .env.settlement-program.example .env.settlement-program

# 编辑配置文件
vim .env.settlement-program
# 或使用你喜欢的编辑器
```

### 4.2 更新配置内容

```bash
# 必须修改的项：
USE_SETTLEMENT_PROGRAM=true
SETTLEMENT_PROGRAM_ID=<刚才部署得到的实际PROGRAM_ID>

# 检查这些项是否正确：
SOLANA_RPC_URL=https://testnet-rpc.1024chain.com/rpc/
SETTLEMENT_AUTHORITY_KEYPAIR=./settlement-authority-fixed.json

# 其他项可以保持默认
```

### 4.3 激活配置

```bash
# 复制为主配置文件
cp .env.settlement-program .env

# 验证配置
cat .env | grep SETTLEMENT
```

---

## 🧪 步骤5: 测试验证

### 5.1 启动Backend

```bash
# 确保在1024-core目录
cd /Users/chuciqin/Desktop/project1024/1024codebase/1024-core

# 重启服务
make restart

# 或手动启动
cd /Users/chuciqin/Desktop/project1024/1024codebase/1024-core/crates/gateway
cargo run --release
```

### 5.2 查看日志

```bash
# 查看启动日志
tail -f /tmp/backend-debug.log

# 应该看到：
# 🔗 使用Settlement Program Backend (完整trade数据)
# 🔗 SettlementProgramBackend initialized
#    RPC: https://testnet-rpc.1024chain.com/rpc/
#    Program: <PROGRAM_ID>
#    Authority: <Authority公钥>
#    Balance: X.XXXXXX N1024
```

### 5.3 运行撮合测试

```bash
# 在项目根目录
cd /Users/chuciqin/Desktop/project1024/1024codebase

# 运行测试脚本
./test-matching-final.sh

# 或者手动测试
# 1. 下单
# 2. 等待撮合
# 3. 查看settlement
```

### 5.4 验证Settlement数据

#### 方法1: 查看日志

```bash
tail -100 /tmp/backend-debug.log | grep Settlement

# 应该看到：
# 📤 Submitting settlement to Settlement Program...
#    Batch ID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
#    Trades: 2
# ✅ Settlement recorded successfully!
#    TX: <交易签名>
#    Settlement Account: <PDA地址>
```

#### 方法2: 查询链上数据

```bash
# 使用日志中的PDA地址
solana account <PDA地址> \
  --url https://testnet-rpc.1024chain.com/rpc/

# 应该显示：
# Account: <PDA地址>
# Owner: <PROGRAM_ID>
# Data Length: XXXX bytes
# Rent Exempt: Yes
```

#### 方法3: 区块浏览器

访问: `https://testnet-scan.1024chain.com/account/<PDA地址>`

---

## ✅ 验收标准

部署成功的标志：

1. ✅ Program成功部署到1024Chain testnet
2. ✅ Backend启动时显示Settlement Program配置
3. ✅ Authority账户有足够余额（>0.01 N1024）
4. ✅ 测试交易生成settlement
5. ✅ Settlement account在链上可查
6. ✅ Account data包含完整trade信息
7. ✅ 区块浏览器可查看settlement

---

## 🔍 故障排查

### 问题1: BPF编译失败

**错误**: `error: failed to download base64ct`

**解决**:
```bash
rustup update
# 或
rustup default nightly
cargo clean
cargo build-sbf
```

---

### 问题2: 部署失败

**错误**: `Error: Insufficient balance`

**解决**:
```bash
# 检查余额
solana balance

# 如果不足，申请测试代币
# （联系1024Chain团队）
```

---

### 问题3: Backend无法连接Program

**错误**: `Program account not found`

**解决**:
1. 检查SETTLEMENT_PROGRAM_ID是否正确
2. 检查Program是否成功部署
   ```bash
   solana program show <PROGRAM_ID>
   ```
3. 检查网络配置是否正确

---

### 问题4: Authority签名失败

**错误**: `Invalid authority - not authorized relayer`

**解决**:
1. 检查keypair文件路径是否正确
2. 检查Authority公钥是否与Program中硬编码的一致
3. 查看`programs/settlement-program/src/processor.rs`第24行

---

### 问题5: 租金不足

**错误**: `Insufficient lamports for rent`

**解决**:
```bash
# 检查Authority余额
solana balance

# 每个settlement account需要约0.01 N1024租金
# 如果余额不足，申请更多测试代币
```

---

## 📚 参考文档

- **项目交接**: `🎁交接文档.md`
- **实施完成报告**: `✅实施完成报告.md`
- **完整Trade定义**: `完整Trade信息定义.md`
- **数据结构设计**: `02-数据结构设计.md`
- **Program架构**: `03-Program架构.md`

---

## 🎯 下一步

部署完成后：

1. **监控**: 设置日志监控，观察settlement频率和成功率
2. **性能**: 记录每个settlement的数据大小和gas费用
3. **优化**: 根据实际使用情况调整批次大小和提交频率
4. **扩展**: 考虑扩展`common::Trade`结构，添加完整的账户信息

---

## 💡 最佳实践

### 生产环境建议

1. **备份Keypair**: 
   ```bash
   cp settlement-authority-fixed.json settlement-authority-backup.json
   chmod 400 settlement-authority-backup.json
   ```

2. **监控余额**:
   - 设置自动告警，当余额<0.1 N1024时通知
   - 定期充值

3. **日志管理**:
   - 保留settlement日志至少30天
   - 定期分析失败原因

4. **升级策略**:
   - Program支持升级（使用BPFLoaderUpgradeable）
   - 升级前在testnet充分测试
   - 保留旧版本rollback计划

---

**部署指南完成！** 🚀

**时间**: 2025年11月12日  
**适用版本**: Settlement Program v1.0

