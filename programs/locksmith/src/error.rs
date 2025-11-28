use shank::ShankType;
use solana_program::program_error::ProgramError;

/// Locksmith program errors
#[derive(Debug, Copy, Clone, PartialEq, Eq, ShankType)]
#[repr(u32)]
pub enum LocksmithError {
    /// Caller is not authorized to perform this action
    Unauthorized = 0,
    /// Unlock timestamp must be in the future
    InvalidTimestamp,
    /// Insufficient token balance for this operation
    InsufficientFunds,
    /// Cannot unlock tokens before the unlock timestamp
    UnlockTooEarly,
    /// Lock token amount doesn't match lock account amount
    InconsistentState,
    /// Lock amount must be greater than zero
    InvalidAmount,
    /// Invalid instruction data
    InvalidInstruction,
    /// Account has not been initialized
    UninitializedAccount,
    /// Account has already been initialized
    AlreadyInitialized,
    /// Invalid PDA derivation
    InvalidPDA,
    /// Invalid token mint
    InvalidMint,
    /// Lock duration exceeds maximum of 10 years
    LockDurationExceeded,
}

impl From<LocksmithError> for ProgramError {
    fn from(e: LocksmithError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Documents the ABI contract - error codes must remain stable for client compatibility
    #[test]
    fn test_error_codes_are_sequential_and_start_at_zero() {
        assert_eq!(LocksmithError::Unauthorized as u32, 0);
        assert_eq!(LocksmithError::InvalidTimestamp as u32, 1);
        assert_eq!(LocksmithError::InsufficientFunds as u32, 2);
        assert_eq!(LocksmithError::UnlockTooEarly as u32, 3);
        assert_eq!(LocksmithError::InconsistentState as u32, 4);
        assert_eq!(LocksmithError::InvalidAmount as u32, 5);
        assert_eq!(LocksmithError::InvalidInstruction as u32, 6);
        assert_eq!(LocksmithError::UninitializedAccount as u32, 7);
        assert_eq!(LocksmithError::AlreadyInitialized as u32, 8);
        assert_eq!(LocksmithError::InvalidPDA as u32, 9);
        assert_eq!(LocksmithError::InvalidMint as u32, 10);
        assert_eq!(LocksmithError::LockDurationExceeded as u32, 11);
    }

    /// Tests the From<LocksmithError> for ProgramError conversion
    #[test]
    fn test_error_to_program_error_conversion() {
        let error = LocksmithError::Unauthorized;
        let program_error: ProgramError = error.into();
        assert_eq!(program_error, ProgramError::Custom(0));

        let error = LocksmithError::InvalidMint;
        let program_error: ProgramError = error.into();
        assert_eq!(program_error, ProgramError::Custom(10));
    }
}
