//! Settlement Program Error Types

use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementError {
    #[error("Invalid settlement account")]
    InvalidSettlementAccount = 0,
    
    #[error("Invalid authority - not authorized relayer")]
    InvalidAuthority = 1,
    
    #[error("Empty trades - batch must contain at least one trade")]
    EmptyTrades = 2,
    
    #[error("Invalid total volume - calculated volume does not match provided")]
    InvalidTotalVolume = 3,
    
    #[error("Invalid total fees - calculated fees do not match provided")]
    InvalidTotalFees = 4,
    
    #[error("Invalid data hash - hash verification failed")]
    InvalidDataHash = 5,
    
    #[error("Account already exists - batch_id already recorded")]
    AccountAlreadyExists = 6,
    
    #[error("Account not found")]
    AccountNotFound = 7,
    
    #[error("Insufficient lamports for rent")]
    InsufficientLamports = 8,
    
    #[error("Invalid batch ID format")]
    InvalidBatchId = 9,
    
    #[error("Serialization error")]
    SerializationError = 10,
    
    #[error("Invalid trade data")]
    InvalidTradeData = 11,
    
    #[error("Invalid trade - price, qty, or notional mismatch")]
    InvalidTrade = 12,
}

impl From<SettlementError> for ProgramError {
    fn from(e: SettlementError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<std::io::Error> for SettlementError {
    fn from(_: std::io::Error) -> Self {
        SettlementError::SerializationError
    }
}

