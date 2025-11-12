//! Settlement Program Instruction Processing

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::SettlementError,
    instruction::SettlementInstruction,
    state::{CompleteTrade, UserSettlement},
    utils::validate_settlement_data,
};

/// 授权的Relayer公钥（临时硬编码，未来从Config读取）
pub const AUTHORIZED_RELAYER: &str = "J1Szw8HZYL95NvYUsNhg3e6NzKQLUZ9UxQsKg4hsQnad";

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = SettlementInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    match instruction {
        SettlementInstruction::InitializeUser { wallet } => {
            msg!("Instruction: InitializeUser");
            process_initialize_user(program_id, accounts, wallet)
        }
        SettlementInstruction::RecordSettlement { batch_id, trades } => {
            msg!("Instruction: RecordSettlement");
            process_record_settlement(program_id, accounts, batch_id, trades)
        }
    }
}

/// 初始化用户的Settlement账户（首次交易时调用）
fn process_initialize_user(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    wallet: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let user_settlement_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;
    
    msg!("Initializing UserSettlement for: {}", wallet);
    
    // 验证authority签名
    if !authority.is_signer {
        msg!("Error: Authority is not signer");
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // 验证authority是授权的relayer
    let authorized_key = AUTHORIZED_RELAYER.parse::<Pubkey>()
        .map_err(|_| SettlementError::InvalidAuthority)?;
    
    if authority.key != &authorized_key {
        msg!("Error: Authority {} is not authorized relayer {}", 
            authority.key, authorized_key);
        return Err(SettlementError::InvalidAuthority.into());
    }
    
    // 派生UserSettlement PDA
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[b"user_settlement", wallet.as_ref()],
        program_id,
    );
    
    if user_settlement_account.key != &expected_pda {
        msg!("Error: PDA mismatch. Expected {}, got {}", 
            expected_pda, user_settlement_account.key);
        return Err(SettlementError::InvalidSettlementAccount.into());
    }
    
    // 验证account尚不存在
    if user_settlement_account.lamports() > 0 {
        msg!("Error: UserSettlement already exists");
        return Err(SettlementError::AccountAlreadyExists.into());
    }
    
    // 计算租金
    let rent = Rent::get()?;
    let space = UserSettlement::SIZE;
    let required_lamports = rent.minimum_balance(space);
    
    msg!("Creating UserSettlement PDA...");
    msg!("  Space: {} bytes", space);
    msg!("  Rent: {} lamports", required_lamports);
    
    // 创建account（通过CPI调用System Program）
    use solana_program::system_instruction;
    let create_account_ix = system_instruction::create_account(
        authority.key,
        user_settlement_account.key,
        required_lamports,
        space as u64,
        program_id,
    );
    
    invoke_signed(
        &create_account_ix,
        &[
            authority.clone(),
            user_settlement_account.clone(),
            system_program.clone(),
        ],
        &[&[b"user_settlement", wallet.as_ref(), &[bump]]],
    )?;
    
    // 初始化数据（空统计）
    let now = solana_program::clock::Clock::get()?.unix_timestamp * 1000;
    let user_settlement = UserSettlement::new(wallet, bump, now);
    
    // 序列化并写入
    let serialized = user_settlement.try_to_vec()
        .map_err(|_| SettlementError::SerializationError)?;
    
    user_settlement_account.data.borrow_mut().copy_from_slice(&serialized);
    
    msg!("✅ UserSettlement initialized for {}", wallet);
    
    Ok(())
}

/// 记录Settlement并更新用户统计
fn process_record_settlement(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    batch_id: String,
    trades: Vec<CompleteTrade>,
) -> ProgramResult {
    msg!("Recording settlement...");
    msg!("  Batch ID: {}", batch_id);
    msg!("  Trades: {}", trades.len());
    
    // 最后一个account是authority
    let accounts_list: Vec<_> = accounts.iter().collect();
    let authority = accounts_list.last()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;
    
    // 验证authority
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    let authorized_key = AUTHORIZED_RELAYER.parse::<Pubkey>()
        .map_err(|_| SettlementError::InvalidAuthority)?;
    
    if authority.key != &authorized_key {
        return Err(SettlementError::InvalidAuthority.into());
    }
    
    // 验证trades数据
    validate_settlement_data(&trades)?;
    
    // Emit settlement开始日志
    msg!("SETTLEMENT_START|batch_id:{}|trades:{}|timestamp:{}", 
        batch_id, trades.len(), 
        solana_program::clock::Clock::get()?.unix_timestamp * 1000);
    
    // 收集需要更新的用户PDAs
    // 格式：(wallet, pda, is_taker, trade_index)
    let mut user_updates: Vec<(Pubkey, Pubkey, bool, usize)> = Vec::new();
    
    for (idx, trade) in trades.iter().enumerate() {
        // Taker PDA
        let (taker_pda, _) = Pubkey::find_program_address(
            &[b"user_settlement", trade.taker_wallet.as_ref()],
            program_id,
        );
        user_updates.push((trade.taker_wallet, taker_pda, true, idx));
        
        // Maker PDA  
        let (maker_pda, _) = Pubkey::find_program_address(
            &[b"user_settlement", trade.maker_wallet.as_ref()],
            program_id,
        );
        user_updates.push((trade.maker_wallet, maker_pda, false, idx));
    }
    
    // 更新每个UserSettlement account
    let mut account_idx = 0;
    for (wallet, expected_pda, is_taker, trade_idx) in user_updates {
        if account_idx >= accounts.len() - 1 {
            msg!("Error: Not enough accounts provided");
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        
        let user_account = &accounts[account_idx];
        account_idx += 1;
        
        // 验证PDA正确
        if user_account.key != &expected_pda {
            msg!("Error: UserSettlement PDA mismatch for wallet {}", wallet);
            msg!("  Expected: {}, Got: {}", expected_pda, user_account.key);
            return Err(SettlementError::InvalidSettlementAccount.into());
        }
        
        // 验证account owner
        if user_account.owner != program_id {
            msg!("Error: Account owner mismatch");
            return Err(ProgramError::IllegalOwner);
        }
        
        // 读取、更新、写回UserSettlement数据
        let mut user_settlement = UserSettlement::try_from_slice(&user_account.data.borrow())
            .map_err(|_| SettlementError::SerializationError)?;
        
        let trade = &trades[trade_idx];
        
        if is_taker {
            user_settlement.update_as_taker(trade);
            msg!("  Updated taker: {} (trades: {})", wallet, user_settlement.total_trades);
        } else {
            user_settlement.update_as_maker(trade);
            msg!("  Updated maker: {} (trades: {})", wallet, user_settlement.total_trades);
        }
        
        // 写回数据
        let serialized = user_settlement.try_to_vec()
            .map_err(|_| SettlementError::SerializationError)?;
        
        user_account.data.borrow_mut().copy_from_slice(&serialized);
    }
    
    // Emit详细的trade logs（所有19字段）
    let mut total_volume_e6: i64 = 0;
    let mut total_fees_e6: i64 = 0;
    
    for trade in &trades {
        let notional = (trade.price_e6 as i128 * trade.qty_e6 as i128 / 1_000_000) as i64;
        total_volume_e6 += notional;
        total_fees_e6 += trade.taker_fee_e6 + trade.maker_fee_e6;
        
        // Trade基础信息
        msg!("TRADE|id:{}|market:{}|price_e6:{}|qty_e6:{}|notional_e6:{}|side:{}|ts:{}|seq:{}", 
            trade.id,
            trade.market,
            trade.price_e6,
            trade.qty_e6,
            trade.notional_e6,
            if matches!(trade.taker_side, crate::state::Side::Buy) { "buy" } else { "sell" },
            trade.ts_ms,
            trade.engine_seq
        );
        
        // 订单信息
        msg!("  orders|taker:{}|maker:{}", 
            trade.taker_order_id,
            trade.maker_order_id
        );
        
        // 账户信息
        msg!("  accounts|taker_id:{}|maker_id:{}|taker_wallet:{}|maker_wallet:{}", 
            trade.taker_account_id,
            trade.maker_account_id,
            trade.taker_wallet,
            trade.maker_wallet
        );
        
        // 杠杆信息
        msg!("  leverage|taker:{}x|maker:{}x", 
            trade.taker_leverage,
            trade.maker_leverage
        );
        
        // 手续费信息
        msg!("  fees|taker:{}|maker:{}|taker_rate:{}bp|maker_rate:{}bp", 
            trade.taker_fee_e6,
            trade.maker_fee_e6,
            trade.fee_rate_taker_bp,
            trade.fee_rate_maker_bp
        );
    }
    
    // Emit settlement结束日志
    msg!("SETTLEMENT_END|batch_id:{}|total_volume:{}|total_fees:{}", 
        batch_id, total_volume_e6, total_fees_e6);
    
    msg!("✅ Settlement recorded successfully!");
    msg!("  {} trades processed", trades.len());
    msg!("  {} user accounts updated", account_idx);
    
    Ok(())
}
