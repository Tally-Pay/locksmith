use solana_program::program_error::ProgramError;

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum LocksmithError {
    Unauthorized = 0,
    InvalidTimestamp,
    InsufficientFunds,
    UnlockTooEarly,
    InconsistentState,
    InvalidAmount,
    InvalidInstruction,
    UninitializedAccount,
    AlreadyInitialized,
    InvalidPDA,
    InvalidMint,
}

impl From<LocksmithError> for ProgramError {
    fn from(e: LocksmithError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
