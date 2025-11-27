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

/// Maximum lock duration: 10 years in seconds
/// This prevents accidental permanent locks while supporting all legitimate use cases
pub const MAX_LOCK_DURATION_SECONDS: i64 = 10 * 365 * 24 * 60 * 60;

/// Config account - stores admin and program state
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_config_account_pack_unpack_roundtrip() {
        let config = ConfigAccount {
            discriminator: ConfigAccount::DISCRIMINATOR,
            admin: Pubkey::new_unique(),
            bump: 255,
        };

        let mut buffer = vec![0u8; ConfigAccount::SIZE];
        config.pack(&mut buffer);

        let unpacked = ConfigAccount::unpack(&buffer).unwrap();
        assert_eq!(config, unpacked);
    }

    #[test]
    fn test_config_account_unpack_insufficient_size() {
        let data = vec![0u8; ConfigAccount::SIZE - 1];
        let result = ConfigAccount::unpack(&data);
        assert_eq!(
            result.unwrap_err(),
            ProgramError::Custom(LocksmithError::UninitializedAccount as u32)
        );
    }

    #[test]
    fn test_config_account_unpack_wrong_discriminator() {
        let mut data = vec![0u8; ConfigAccount::SIZE];
        data[0..8].copy_from_slice(b"WRONGDIS");

        let result = ConfigAccount::unpack(&data);
        assert_eq!(
            result.unwrap_err(),
            ProgramError::Custom(LocksmithError::UninitializedAccount as u32)
        );
    }

    #[test]
    fn test_lock_account_pack_unpack_roundtrip() {
        let lock = LockAccount {
            discriminator: LockAccount::DISCRIMINATOR,
            owner: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            amount: 1_000_000_000,
            unlock_timestamp: 1700000000,
            created_at: 1699000000,
            lock_id: 42,
            bump: 254,
        };

        let mut buffer = vec![0u8; LockAccount::SIZE];
        lock.pack(&mut buffer);

        let unpacked = LockAccount::unpack(&buffer).unwrap();
        assert_eq!(lock, unpacked);
    }

    #[test]
    fn test_lock_account_unpack_insufficient_size() {
        let data = vec![0u8; LockAccount::SIZE - 1];
        let result = LockAccount::unpack(&data);
        assert_eq!(
            result.unwrap_err(),
            ProgramError::Custom(LocksmithError::UninitializedAccount as u32)
        );
    }

    #[test]
    fn test_lock_account_unpack_wrong_discriminator() {
        let mut data = vec![0u8; LockAccount::SIZE];
        data[0..8].copy_from_slice(b"WRONGDIS");

        let result = LockAccount::unpack(&data);
        assert_eq!(
            result.unwrap_err(),
            ProgramError::Custom(LocksmithError::UninitializedAccount as u32)
        );
    }

    #[test]
    fn test_lock_account_rejects_config_discriminator() {
        let mut data = vec![0u8; LockAccount::SIZE];
        data[0..8].copy_from_slice(&ConfigAccount::DISCRIMINATOR);

        assert!(LockAccount::unpack(&data).is_err());
    }

    #[test]
    fn test_discriminators_are_unique() {
        assert_ne!(ConfigAccount::DISCRIMINATOR, LockAccount::DISCRIMINATOR);
    }

    #[test]
    fn test_config_account_byte_layout() {
        let admin_bytes: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let config = ConfigAccount {
            discriminator: ConfigAccount::DISCRIMINATOR,
            admin: Pubkey::from(admin_bytes),
            bump: 200,
        };

        let mut buffer = vec![0u8; ConfigAccount::SIZE];
        config.pack(&mut buffer);

        assert_eq!(&buffer[0..8], b"CONFIG\0\0");
        assert_eq!(&buffer[8..40], &admin_bytes);
        assert_eq!(buffer[40], 200);
    }

    #[test]
    fn test_lock_account_byte_layout() {
        let owner_bytes: [u8; 32] = [1u8; 32];
        let mint_bytes: [u8; 32] = [2u8; 32];

        let lock = LockAccount {
            discriminator: LockAccount::DISCRIMINATOR,
            owner: Pubkey::from(owner_bytes),
            mint: Pubkey::from(mint_bytes),
            amount: 0x0102030405060708,
            unlock_timestamp: 0x090A0B0C0D0E0F10_u64 as i64,
            created_at: 0x1112131415161718_u64 as i64,
            lock_id: 0x191A1B1C1D1E1F20,
            bump: 250,
        };

        let mut buffer = vec![0u8; LockAccount::SIZE];
        lock.pack(&mut buffer);

        assert_eq!(&buffer[0..8], b"LOCK\0\0\0\0");
        assert_eq!(&buffer[8..40], &owner_bytes);
        assert_eq!(&buffer[40..72], &mint_bytes);
        assert_eq!(u64::from_le_bytes(buffer[72..80].try_into().unwrap()), 0x0102030405060708);
        assert_eq!(i64::from_le_bytes(buffer[80..88].try_into().unwrap()), 0x090A0B0C0D0E0F10_u64 as i64);
        assert_eq!(i64::from_le_bytes(buffer[88..96].try_into().unwrap()), 0x1112131415161718_u64 as i64);
        assert_eq!(u64::from_le_bytes(buffer[96..104].try_into().unwrap()), 0x191A1B1C1D1E1F20);
        assert_eq!(buffer[104], 250);
    }

    #[test]
    fn test_max_lock_duration_constant() {
        // 10 years = 10 * 365 * 24 * 60 * 60 seconds
        assert_eq!(MAX_LOCK_DURATION_SECONDS, 315_360_000);
    }

    #[test]
    fn test_max_lock_duration_is_positive() {
        assert!(MAX_LOCK_DURATION_SECONDS > 0);
    }
}
