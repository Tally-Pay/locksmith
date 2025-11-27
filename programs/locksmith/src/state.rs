use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::LocksmithError;

/// Seeds for PDA derivation
pub const CONFIG_SEED: &[u8] = b"config";
pub const FEE_VAULT_SEED: &[u8] = b"fee_vault";
pub const LOCK_SEED: &[u8] = b"lock";
pub const LOCK_TOKEN_SEED: &[u8] = b"lock_token";

/// USDC mint address on mainnet
pub const USDC_MINT: Pubkey =
    solana_program::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

/// Fee amount: 0.15 USDC (USDC has 6 decimals)
pub const FEE_USDC: u64 = 150_000;

/// Config account - stores admin and program state
#[derive(Debug)]
pub struct ConfigAccount {
    pub discriminator: [u8; 8],
    pub admin: Pubkey,
    pub bump: u8,
}

impl ConfigAccount {
    pub const DISCRIMINATOR: [u8; 8] = *b"CONFIG\0\0";
    pub const SIZE: usize = 8 + 32 + 1;

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(LocksmithError::UninitializedAccount.into());
        }
        let discriminator: [u8; 8] = data[0..8].try_into().unwrap();
        if discriminator != Self::DISCRIMINATOR {
            return Err(LocksmithError::UninitializedAccount.into());
        }
        let admin = Pubkey::try_from(&data[8..40]).unwrap();
        let bump = data[40];
        Ok(Self {
            discriminator,
            admin,
            bump,
        })
    }

    pub fn pack(&self, dst: &mut [u8]) {
        dst[0..8].copy_from_slice(&self.discriminator);
        dst[8..40].copy_from_slice(self.admin.as_ref());
        dst[40] = self.bump;
    }
}

/// Lock account - stores information about a single token lock
#[derive(Debug)]
pub struct LockAccount {
    pub discriminator: [u8; 8],
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub unlock_timestamp: i64,
    pub created_at: i64,
    pub lock_id: u64,
    pub bump: u8,
}

impl LockAccount {
    pub const DISCRIMINATOR: [u8; 8] = *b"LOCK\0\0\0\0";
    pub const SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 1;

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(LocksmithError::UninitializedAccount.into());
        }
        let discriminator: [u8; 8] = data[0..8].try_into().unwrap();
        if discriminator != Self::DISCRIMINATOR {
            return Err(LocksmithError::UninitializedAccount.into());
        }
        let owner = Pubkey::try_from(&data[8..40]).unwrap();
        let mint = Pubkey::try_from(&data[40..72]).unwrap();
        let amount = u64::from_le_bytes(data[72..80].try_into().unwrap());
        let unlock_timestamp = i64::from_le_bytes(data[80..88].try_into().unwrap());
        let created_at = i64::from_le_bytes(data[88..96].try_into().unwrap());
        let lock_id = u64::from_le_bytes(data[96..104].try_into().unwrap());
        let bump = data[104];
        Ok(Self {
            discriminator,
            owner,
            mint,
            amount,
            unlock_timestamp,
            created_at,
            lock_id,
            bump,
        })
    }

    pub fn pack(&self, dst: &mut [u8]) {
        dst[0..8].copy_from_slice(&self.discriminator);
        dst[8..40].copy_from_slice(self.owner.as_ref());
        dst[40..72].copy_from_slice(self.mint.as_ref());
        dst[72..80].copy_from_slice(&self.amount.to_le_bytes());
        dst[80..88].copy_from_slice(&self.unlock_timestamp.to_le_bytes());
        dst[88..96].copy_from_slice(&self.created_at.to_le_bytes());
        dst[96..104].copy_from_slice(&self.lock_id.to_le_bytes());
        dst[104] = self.bump;
    }
}
