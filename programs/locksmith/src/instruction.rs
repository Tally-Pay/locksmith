use solana_program::program_error::ProgramError;

use crate::error::LocksmithError;

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // INSTRUCTION PARSING TESTS
    // ============================================================================

    #[test]
    fn test_unpack_initialize_config() {
        let data = [0u8];
        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(instruction, LocksmithInstruction::InitializeConfig);
    }

    #[test]
    fn test_unpack_transfer_admin() {
        let data = [1u8];
        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(instruction, LocksmithInstruction::TransferAdmin);
    }

    #[test]
    fn test_unpack_withdraw_fees() {
        let data = [2u8];
        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(instruction, LocksmithInstruction::WithdrawFees);
    }

    #[test]
    fn test_unpack_initialize_lock() {
        let amount: u64 = 1_000_000;
        let unlock_timestamp: i64 = 1700000000;
        let lock_id: u64 = 42;

        let mut data = vec![3u8];
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(&unlock_timestamp.to_le_bytes());
        data.extend_from_slice(&lock_id.to_le_bytes());

        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(
            instruction,
            LocksmithInstruction::InitializeLock {
                amount,
                unlock_timestamp,
                lock_id
            }
        );
    }

    #[test]
    fn test_unpack_unlock() {
        let lock_id: u64 = 42;

        let mut data = vec![4u8];
        data.extend_from_slice(&lock_id.to_le_bytes());

        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(instruction, LocksmithInstruction::Unlock { lock_id });
    }

    // ============================================================================
    // SECURITY: INPUT VALIDATION & BOUNDARY TESTS
    // ============================================================================

    #[test]
    fn test_unpack_empty_data_returns_error() {
        let data: [u8; 0] = [];
        let result = LocksmithInstruction::unpack(&data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ProgramError::Custom(LocksmithError::InvalidInstruction as u32)
        );
    }

    #[test]
    fn test_unpack_invalid_tag_returns_error() {
        // Test all invalid tags
        for invalid_tag in [5u8, 6, 100, 255] {
            let data = [invalid_tag];
            let result = LocksmithInstruction::unpack(&data);
            assert!(
                result.is_err(),
                "Tag {} should return error",
                invalid_tag
            );
            assert_eq!(
                result.unwrap_err(),
                ProgramError::Custom(LocksmithError::InvalidInstruction as u32)
            );
        }
    }

    #[test]
    fn test_unpack_initialize_lock_insufficient_data() {
        // Tag 3 requires 24 bytes of data (amount + unlock_timestamp + lock_id)
        let test_cases = [
            vec![3u8],                            // 0 bytes
            vec![3u8, 0, 0, 0, 0, 0, 0, 0],       // 7 bytes (need 24)
            vec![3u8, 0, 0, 0, 0, 0, 0, 0, 0],    // 8 bytes
            vec![3u8; 17],                        // 16 bytes
            vec![3u8; 24],                        // 23 bytes (one short)
        ];

        for data in test_cases {
            let result = LocksmithInstruction::unpack(&data);
            assert!(
                result.is_err(),
                "Data of length {} should fail for InitializeLock",
                data.len() - 1
            );
        }
    }

    #[test]
    fn test_unpack_unlock_insufficient_data() {
        // Tag 4 requires 8 bytes of data (lock_id)
        let test_cases = [
            vec![4u8],                         // 0 bytes
            vec![4u8, 0, 0, 0],                // 3 bytes
            vec![4u8, 0, 0, 0, 0, 0, 0, 0],    // 7 bytes (one short)
        ];

        for data in test_cases {
            let result = LocksmithInstruction::unpack(&data);
            assert!(
                result.is_err(),
                "Data of length {} should fail for Unlock",
                data.len() - 1
            );
        }
    }

    // ============================================================================
    // SECURITY: BOUNDARY VALUE TESTS (POTENTIAL OVERFLOW/UNDERFLOW)
    // ============================================================================

    #[test]
    fn test_unpack_initialize_lock_max_values() {
        let amount: u64 = u64::MAX;
        let unlock_timestamp: i64 = i64::MAX;
        let lock_id: u64 = u64::MAX;

        let mut data = vec![3u8];
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(&unlock_timestamp.to_le_bytes());
        data.extend_from_slice(&lock_id.to_le_bytes());

        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(
            instruction,
            LocksmithInstruction::InitializeLock {
                amount,
                unlock_timestamp,
                lock_id
            }
        );
    }

    #[test]
    fn test_unpack_initialize_lock_min_values() {
        let amount: u64 = 0;
        let unlock_timestamp: i64 = i64::MIN;
        let lock_id: u64 = 0;

        let mut data = vec![3u8];
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(&unlock_timestamp.to_le_bytes());
        data.extend_from_slice(&lock_id.to_le_bytes());

        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(
            instruction,
            LocksmithInstruction::InitializeLock {
                amount,
                unlock_timestamp,
                lock_id
            }
        );
    }

    #[test]
    fn test_unpack_initialize_lock_negative_timestamp() {
        let amount: u64 = 1000;
        let unlock_timestamp: i64 = -1; // Before Unix epoch
        let lock_id: u64 = 1;

        let mut data = vec![3u8];
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(&unlock_timestamp.to_le_bytes());
        data.extend_from_slice(&lock_id.to_le_bytes());

        // Parsing should succeed - validation happens in processor
        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(
            instruction,
            LocksmithInstruction::InitializeLock {
                amount,
                unlock_timestamp,
                lock_id
            }
        );
    }

    #[test]
    fn test_unpack_unlock_max_lock_id() {
        let lock_id: u64 = u64::MAX;

        let mut data = vec![4u8];
        data.extend_from_slice(&lock_id.to_le_bytes());

        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(instruction, LocksmithInstruction::Unlock { lock_id });
    }

    #[test]
    fn test_unpack_unlock_zero_lock_id() {
        let lock_id: u64 = 0;

        let mut data = vec![4u8];
        data.extend_from_slice(&lock_id.to_le_bytes());

        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(instruction, LocksmithInstruction::Unlock { lock_id });
    }

    // ============================================================================
    // SECURITY: EXTRA DATA HANDLING
    // ============================================================================

    #[test]
    fn test_unpack_initialize_config_ignores_extra_data() {
        // Extra data after a valid instruction should be ignored
        let data = [0u8, 0xFF, 0xFF, 0xFF, 0xFF];
        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(instruction, LocksmithInstruction::InitializeConfig);
    }

    #[test]
    fn test_unpack_initialize_lock_ignores_extra_data() {
        let amount: u64 = 1000;
        let unlock_timestamp: i64 = 1700000000;
        let lock_id: u64 = 1;

        let mut data = vec![3u8];
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(&unlock_timestamp.to_le_bytes());
        data.extend_from_slice(&lock_id.to_le_bytes());
        // Add extra garbage data
        data.extend_from_slice(&[0xFF; 100]);

        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(
            instruction,
            LocksmithInstruction::InitializeLock {
                amount,
                unlock_timestamp,
                lock_id
            }
        );
    }

    #[test]
    fn test_unpack_unlock_ignores_extra_data() {
        let lock_id: u64 = 42;

        let mut data = vec![4u8];
        data.extend_from_slice(&lock_id.to_le_bytes());
        // Add extra garbage data
        data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        assert_eq!(instruction, LocksmithInstruction::Unlock { lock_id });
    }

    // ============================================================================
    // ENDIANNESS TESTS
    // ============================================================================

    #[test]
    fn test_unpack_initialize_lock_little_endian() {
        // Explicitly test little-endian byte ordering
        // Amount: 0x0102030405060708 in little-endian = [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        let data: Vec<u8> = vec![
            3u8, // tag
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // amount (little-endian)
            0x10, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, 0x09, // timestamp (little-endian)
            0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11, // lock_id (little-endian)
        ];

        let instruction = LocksmithInstruction::unpack(&data).unwrap();
        match instruction {
            LocksmithInstruction::InitializeLock {
                amount,
                unlock_timestamp,
                lock_id,
            } => {
                assert_eq!(amount, 0x0102030405060708);
                assert_eq!(unlock_timestamp, 0x090A0B0C0D0E0F10_u64 as i64);
                assert_eq!(lock_id, 0x1112131415161718);
            }
            _ => panic!("Expected InitializeLock instruction"),
        }
    }
}
