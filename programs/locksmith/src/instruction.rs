use solana_program::program_error::ProgramError;

use crate::error::LocksmithError;

#[derive(Debug)]
pub enum LocksmithInstruction {
    /// Initialize the program configuration and fee vault.
    InitializeConfig,

    /// Transfer admin role to a new wallet.
    TransferAdmin,

    /// Withdraw accumulated USDC fees to admin's wallet.
    WithdrawFees,

    /// Create a new token lock.
    InitializeLock {
        amount: u64,
        unlock_timestamp: i64,
        lock_id: u64,
    },

    /// Unlock tokens after the unlock timestamp has passed.
    Unlock { lock_id: u64 },
}

impl LocksmithInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input
            .split_first()
            .ok_or(LocksmithError::InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitializeConfig,
            1 => Self::TransferAdmin,
            2 => Self::WithdrawFees,
            3 => {
                if rest.len() < 24 {
                    return Err(LocksmithError::InvalidInstruction.into());
                }
                let amount = u64::from_le_bytes(rest[0..8].try_into().unwrap());
                let unlock_timestamp = i64::from_le_bytes(rest[8..16].try_into().unwrap());
                let lock_id = u64::from_le_bytes(rest[16..24].try_into().unwrap());
                Self::InitializeLock {
                    amount,
                    unlock_timestamp,
                    lock_id,
                }
            }
            4 => {
                if rest.len() < 8 {
                    return Err(LocksmithError::InvalidInstruction.into());
                }
                let lock_id = u64::from_le_bytes(rest[0..8].try_into().unwrap());
                Self::Unlock { lock_id }
            }
            _ => return Err(LocksmithError::InvalidInstruction.into()),
        })
    }
}
