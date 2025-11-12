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
    state::{SettlementAccount, SettlementData, SettlementStatus},
    utils::{validate_batch_id, validate_settlement_data},
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
        SettlementInstruction::RecordSettlement { data } => {
            msg!("Instruction: RecordSettlement");
            process_record_settlement(program_id, accounts, data)
        }
        SettlementInstruction::UpdateSettlement { status } => {
            msg!("Instruction: UpdateSettlement");
            process_update_settlement(program_id, accounts, status)
        }
    }
}

fn process_record_settlement(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: SettlementData,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    // 解析accounts
    let settlement_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;
    
    msg!("Settlement batch: {}", data.batch_id);
    msg!("Trades count: {}", data.trades.len());
    
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
    
    // 验证batch_id格式
    validate_batch_id(&data.batch_id)?;
    
    // 验证settlement account是正确的PDA
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[b"settlement", data.batch_id.as_bytes()],
        program_id,
    );
    
    if settlement_account.key != &expected_pda {
        msg!("Error: Settlement account mismatch. Expected {}, got {}", 
            expected_pda, settlement_account.key);
        return Err(SettlementError::InvalidSettlementAccount.into());
    }
    
    // 验证account不存在（防止重复）
    if settlement_account.lamports() > 0 {
        msg!("Error: Settlement account already exists");
        return Err(SettlementError::AccountAlreadyExists.into());
    }
    
    // 验证settlement数据
    msg!("Validating settlement data...");
    validate_settlement_data(&data)?;
    msg!("✅ Validation passed");
    
    // 创建settlement account数据
    let account_data = SettlementAccount {
        discriminator: SettlementAccount::DISCRIMINATOR,
        version: SettlementAccount::VERSION,
        bump,
        reserved: [0; 6],
        data,
    };
    
    // 序列化
    let serialized = account_data.try_to_vec()
        .map_err(|_| SettlementError::SerializationError)?;
    
    msg!("Settlement data size: {} bytes", serialized.len());
    
    // 计算所需租金
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(serialized.len());
    
    msg!("Required rent: {} lamports", required_lamports);
    
    // 验证authority有足够余额
    if authority.lamports() < required_lamports {
        msg!("Error: Insufficient lamports. Have: {}, Need: {}", 
            authority.lamports(), required_lamports);
        return Err(SettlementError::InsufficientLamports.into());
    }
    
    // 创建settlement account
    msg!("Creating settlement account (allocate + assign)...");
    
    // 手动构建create_account instruction
    // System Program ID是固定的
    let system_program_id = solana_program::pubkey!("11111111111111111111111111111111");
    
    // 创建instruction数据
    let mut instruction_data = vec![0u8; 52]; // CreateAccount instruction
    instruction_data[0..4].copy_from_slice(&[0, 0, 0, 0]); // instruction discriminator
    instruction_data[4..12].copy_from_slice(&required_lamports.to_le_bytes());
    instruction_data[12..20].copy_from_slice(&(serialized.len() as u64).to_le_bytes());
    instruction_data[20..52].copy_from_slice(program_id.as_ref());
    
    let ix = solana_program::instruction::Instruction {
        program_id: system_program_id,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(*authority.key, true),
            solana_program::instruction::AccountMeta::new(*settlement_account.key, false),
        ],
        data: instruction_data,
    };
    
    invoke_signed(
        &ix,
        &[
            authority.clone(),
            settlement_account.clone(),
            system_program.clone(),
        ],
        &[&[b"settlement", account_data.data.batch_id.as_bytes(), &[bump]]],
    )?;
    
    // 写入数据
    let mut settlement_data = settlement_account.data.borrow_mut();
    settlement_data[..serialized.len()].copy_from_slice(&serialized);
    
    msg!("✅ Settlement recorded successfully!");
    msg!("   Batch: {}", account_data.data.batch_id);
    msg!("   Trades: {}", account_data.data.trades.len());
    msg!("   Account: {}", settlement_account.key);
    
    Ok(())
}

fn process_update_settlement(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _status: SettlementStatus,
) -> ProgramResult {
    // 预留功能，暂不实现
    msg!("UpdateSettlement: Not implemented yet");
    Ok(())
}

