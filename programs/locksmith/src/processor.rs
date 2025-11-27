use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use solana_system_interface::instruction as system_instruction;
use spl_token::state::Account as TokenAccount;

use crate::error::LocksmithError;
use crate::instruction::LocksmithInstruction;
use crate::state::{
    ConfigAccount, LockAccount, CONFIG_SEED, FEE_USDC, FEE_VAULT_SEED, LOCK_SEED, LOCK_TOKEN_SEED,
    MAX_LOCK_DURATION_SECONDS, USDC_MINT,
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = LocksmithInstruction::unpack(instruction_data)?;

    match instruction {
        LocksmithInstruction::InitializeConfig => process_initialize_config(program_id, accounts),
        LocksmithInstruction::TransferAdmin => process_transfer_admin(program_id, accounts),
        LocksmithInstruction::WithdrawFees => process_withdraw_fees(program_id, accounts),
        LocksmithInstruction::InitializeLock {
            amount,
            unlock_timestamp,
            lock_id,
        } => process_initialize_lock(program_id, accounts, amount, unlock_timestamp, lock_id),
        LocksmithInstruction::Unlock { lock_id } => process_unlock(program_id, accounts, lock_id),
    }
}

fn process_initialize_config(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let admin_info = next_account_info(account_info_iter)?;
    let config_info = next_account_info(account_info_iter)?;
    let usdc_mint_info = next_account_info(account_info_iter)?;
    let fee_vault_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate token program is the official SPL Token program
    if *token_program_info.key != spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Validate system program is the official System program
    if !solana_system_interface::program::check_id(system_program_info.key) {
        return Err(ProgramError::IncorrectProgramId);
    }

    if *usdc_mint_info.key != USDC_MINT {
        return Err(LocksmithError::InvalidMint.into());
    }

    let (config_pda, config_bump) = Pubkey::find_program_address(&[CONFIG_SEED], program_id);
    if *config_info.key != config_pda {
        return Err(LocksmithError::InvalidPDA.into());
    }

    let (fee_vault_pda, fee_vault_bump) =
        Pubkey::find_program_address(&[FEE_VAULT_SEED], program_id);
    if *fee_vault_info.key != fee_vault_pda {
        return Err(LocksmithError::InvalidPDA.into());
    }

    if !config_info.data_is_empty() {
        return Err(LocksmithError::AlreadyInitialized.into());
    }

    let rent = Rent::get()?;

    invoke_signed(
        &system_instruction::create_account(
            admin_info.key,
            config_info.key,
            rent.minimum_balance(ConfigAccount::SIZE),
            ConfigAccount::SIZE as u64,
            program_id,
        ),
        &[
            admin_info.clone(),
            config_info.clone(),
            system_program_info.clone(),
        ],
        &[&[CONFIG_SEED, &[config_bump]]],
    )?;

    let config = ConfigAccount {
        discriminator: ConfigAccount::DISCRIMINATOR,
        admin: *admin_info.key,
        bump: config_bump,
    };
    config.pack(&mut config_info.data.borrow_mut());

    invoke_signed(
        &system_instruction::create_account(
            admin_info.key,
            fee_vault_info.key,
            rent.minimum_balance(TokenAccount::LEN),
            TokenAccount::LEN as u64,
            &spl_token::id(),
        ),
        &[
            admin_info.clone(),
            fee_vault_info.clone(),
            system_program_info.clone(),
        ],
        &[&[FEE_VAULT_SEED, &[fee_vault_bump]]],
    )?;

    invoke(
        &spl_token::instruction::initialize_account3(
            &spl_token::id(),
            fee_vault_info.key,
            usdc_mint_info.key,
            fee_vault_info.key,
        )?,
        &[fee_vault_info.clone(), usdc_mint_info.clone()],
    )?;

    msg!("Config initialized with admin: {}", admin_info.key);
    Ok(())
}

fn process_transfer_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let admin_info = next_account_info(account_info_iter)?;
    let new_admin_info = next_account_info(account_info_iter)?;
    let config_info = next_account_info(account_info_iter)?;

    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (config_pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], program_id);
    if *config_info.key != config_pda {
        return Err(LocksmithError::InvalidPDA.into());
    }

    let mut config = ConfigAccount::unpack(&config_info.data.borrow())?;

    if config.admin != *admin_info.key {
        return Err(LocksmithError::Unauthorized.into());
    }

    let old_admin = config.admin;
    config.admin = *new_admin_info.key;
    config.pack(&mut config_info.data.borrow_mut());

    msg!("Admin transferred from {} to {}", old_admin, new_admin_info.key);
    Ok(())
}

fn process_withdraw_fees(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let admin_info = next_account_info(account_info_iter)?;
    let config_info = next_account_info(account_info_iter)?;
    let fee_vault_info = next_account_info(account_info_iter)?;
    let admin_token_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;

    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (config_pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], program_id);
    if *config_info.key != config_pda {
        return Err(LocksmithError::InvalidPDA.into());
    }

    let (fee_vault_pda, fee_vault_bump) =
        Pubkey::find_program_address(&[FEE_VAULT_SEED], program_id);
    if *fee_vault_info.key != fee_vault_pda {
        return Err(LocksmithError::InvalidPDA.into());
    }

    let config = ConfigAccount::unpack(&config_info.data.borrow())?;

    if config.admin != *admin_info.key {
        return Err(LocksmithError::Unauthorized.into());
    }

    // Validate token program is the official SPL Token program
    if *token_program_info.key != spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    let fee_vault = TokenAccount::unpack(&fee_vault_info.data.borrow())?;
    let amount = fee_vault.amount;

    if amount == 0 {
        return Ok(());
    }

    invoke_signed(
        &spl_token::instruction::transfer(
            token_program_info.key,
            fee_vault_info.key,
            admin_token_info.key,
            fee_vault_info.key,
            &[],
            amount,
        )?,
        &[
            fee_vault_info.clone(),
            admin_token_info.clone(),
            fee_vault_info.clone(),
        ],
        &[&[FEE_VAULT_SEED, &[fee_vault_bump]]],
    )?;

    msg!("Withdrawn {} USDC to admin", amount);
    Ok(())
}

fn process_initialize_lock(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    unlock_timestamp: i64,
    lock_id: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let owner_info = next_account_info(account_info_iter)?;
    let owner_token_info = next_account_info(account_info_iter)?;
    let owner_usdc_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let lock_account_info = next_account_info(account_info_iter)?;
    let lock_token_info = next_account_info(account_info_iter)?;
    let fee_vault_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if amount == 0 {
        return Err(LocksmithError::InvalidAmount.into());
    }

    // Validate token program is the official SPL Token program
    if *token_program_info.key != spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Validate system program is the official System program
    if !solana_system_interface::program::check_id(system_program_info.key) {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Validate fee vault PDA
    let (fee_vault_pda, _) = Pubkey::find_program_address(&[FEE_VAULT_SEED], program_id);
    if *fee_vault_info.key != fee_vault_pda {
        return Err(LocksmithError::InvalidPDA.into());
    }

    let clock = Clock::get()?;
    if unlock_timestamp <= clock.unix_timestamp {
        return Err(LocksmithError::InvalidTimestamp.into());
    }

    // Validate lock duration does not exceed maximum (10 years)
    let max_unlock_timestamp = clock
        .unix_timestamp
        .checked_add(MAX_LOCK_DURATION_SECONDS)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    if unlock_timestamp > max_unlock_timestamp {
        return Err(LocksmithError::LockDurationExceeded.into());
    }

    let lock_id_bytes = lock_id.to_le_bytes();
    let (lock_pda, lock_bump) = Pubkey::find_program_address(
        &[
            LOCK_SEED,
            owner_info.key.as_ref(),
            mint_info.key.as_ref(),
            &lock_id_bytes,
        ],
        program_id,
    );
    if *lock_account_info.key != lock_pda {
        return Err(LocksmithError::InvalidPDA.into());
    }

    let (lock_token_pda, lock_token_bump) =
        Pubkey::find_program_address(&[LOCK_TOKEN_SEED, lock_account_info.key.as_ref()], program_id);
    if *lock_token_info.key != lock_token_pda {
        return Err(LocksmithError::InvalidPDA.into());
    }

    let owner_token = TokenAccount::unpack(&owner_token_info.data.borrow())?;
    if owner_token.owner != *owner_info.key {
        return Err(LocksmithError::Unauthorized.into());
    }
    if owner_token.mint != *mint_info.key {
        return Err(LocksmithError::InvalidMint.into());
    }
    if owner_token.amount < amount {
        return Err(LocksmithError::InsufficientFunds.into());
    }

    let owner_usdc = TokenAccount::unpack(&owner_usdc_info.data.borrow())?;
    if owner_usdc.owner != *owner_info.key {
        return Err(LocksmithError::Unauthorized.into());
    }
    if owner_usdc.mint != USDC_MINT {
        return Err(LocksmithError::InvalidMint.into());
    }
    if owner_usdc.amount < FEE_USDC {
        return Err(LocksmithError::InsufficientFunds.into());
    }

    let rent = Rent::get()?;

    invoke_signed(
        &system_instruction::create_account(
            owner_info.key,
            lock_account_info.key,
            rent.minimum_balance(LockAccount::SIZE),
            LockAccount::SIZE as u64,
            program_id,
        ),
        &[
            owner_info.clone(),
            lock_account_info.clone(),
            system_program_info.clone(),
        ],
        &[&[
            LOCK_SEED,
            owner_info.key.as_ref(),
            mint_info.key.as_ref(),
            &lock_id_bytes,
            &[lock_bump],
        ]],
    )?;

    let lock = LockAccount {
        discriminator: LockAccount::DISCRIMINATOR,
        owner: *owner_info.key,
        mint: *mint_info.key,
        amount,
        unlock_timestamp,
        created_at: clock.unix_timestamp,
        lock_id,
        bump: lock_bump,
    };
    lock.pack(&mut lock_account_info.data.borrow_mut());

    invoke_signed(
        &system_instruction::create_account(
            owner_info.key,
            lock_token_info.key,
            rent.minimum_balance(TokenAccount::LEN),
            TokenAccount::LEN as u64,
            &spl_token::id(),
        ),
        &[
            owner_info.clone(),
            lock_token_info.clone(),
            system_program_info.clone(),
        ],
        &[&[LOCK_TOKEN_SEED, lock_account_info.key.as_ref(), &[lock_token_bump]]],
    )?;

    invoke(
        &spl_token::instruction::initialize_account3(
            &spl_token::id(),
            lock_token_info.key,
            mint_info.key,
            lock_account_info.key,
        )?,
        &[lock_token_info.clone(), mint_info.clone()],
    )?;

    invoke(
        &spl_token::instruction::transfer(
            token_program_info.key,
            owner_token_info.key,
            lock_token_info.key,
            owner_info.key,
            &[],
            amount,
        )?,
        &[
            owner_token_info.clone(),
            lock_token_info.clone(),
            owner_info.clone(),
        ],
    )?;

    invoke(
        &spl_token::instruction::transfer(
            token_program_info.key,
            owner_usdc_info.key,
            fee_vault_info.key,
            owner_info.key,
            &[],
            FEE_USDC,
        )?,
        &[
            owner_usdc_info.clone(),
            fee_vault_info.clone(),
            owner_info.clone(),
        ],
    )?;

    msg!(
        "Lock created: {} tokens locked until {}",
        amount,
        unlock_timestamp
    );
    Ok(())
}

/// Unlocks tokens after the unlock timestamp has passed.
///
/// # Destination Token Account
///
/// The owner may specify any token account they own (with the correct mint) as the
/// destination for unlocked tokens. This provides flexibility for the lock owner to
/// receive tokens in whichever of their accounts they prefer.
fn process_unlock(program_id: &Pubkey, accounts: &[AccountInfo], lock_id: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let owner_info = next_account_info(account_info_iter)?;
    let owner_token_info = next_account_info(account_info_iter)?;
    let lock_account_info = next_account_info(account_info_iter)?;
    let lock_token_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate token program is the official SPL Token program
    if *token_program_info.key != spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    let lock = LockAccount::unpack(&lock_account_info.data.borrow())?;

    if lock.owner != *owner_info.key {
        return Err(LocksmithError::Unauthorized.into());
    }

    let lock_id_bytes = lock_id.to_le_bytes();
    let (lock_pda, _) = Pubkey::find_program_address(
        &[
            LOCK_SEED,
            owner_info.key.as_ref(),
            lock.mint.as_ref(),
            &lock_id_bytes,
        ],
        program_id,
    );
    if *lock_account_info.key != lock_pda {
        return Err(LocksmithError::InvalidPDA.into());
    }

    let (lock_token_pda, _) =
        Pubkey::find_program_address(&[LOCK_TOKEN_SEED, lock_account_info.key.as_ref()], program_id);
    if *lock_token_info.key != lock_token_pda {
        return Err(LocksmithError::InvalidPDA.into());
    }

    let clock = Clock::get()?;
    if clock.unix_timestamp < lock.unlock_timestamp {
        return Err(LocksmithError::UnlockTooEarly.into());
    }

    let lock_token = TokenAccount::unpack(&lock_token_info.data.borrow())?;
    if lock_token.amount != lock.amount {
        return Err(LocksmithError::InconsistentState.into());
    }

    // Validate destination token account belongs to the owner and has correct mint
    let owner_token = TokenAccount::unpack(&owner_token_info.data.borrow())?;
    if owner_token.owner != *owner_info.key {
        return Err(LocksmithError::Unauthorized.into());
    }
    if owner_token.mint != lock.mint {
        return Err(LocksmithError::InvalidMint.into());
    }

    let amount = lock.amount;
    let lock_bump = lock.bump;

    invoke_signed(
        &spl_token::instruction::transfer(
            token_program_info.key,
            lock_token_info.key,
            owner_token_info.key,
            lock_account_info.key,
            &[],
            amount,
        )?,
        &[
            lock_token_info.clone(),
            owner_token_info.clone(),
            lock_account_info.clone(),
        ],
        &[&[
            LOCK_SEED,
            owner_info.key.as_ref(),
            lock.mint.as_ref(),
            &lock_id_bytes,
            &[lock_bump],
        ]],
    )?;

    invoke_signed(
        &spl_token::instruction::close_account(
            token_program_info.key,
            lock_token_info.key,
            owner_info.key,
            lock_account_info.key,
            &[],
        )?,
        &[
            lock_token_info.clone(),
            owner_info.clone(),
            lock_account_info.clone(),
        ],
        &[&[
            LOCK_SEED,
            owner_info.key.as_ref(),
            lock.mint.as_ref(),
            &lock_id_bytes,
            &[lock_bump],
        ]],
    )?;

    let lock_lamports = lock_account_info.lamports();
    **lock_account_info.lamports.borrow_mut() = 0;
    **owner_info.lamports.borrow_mut() = owner_info
        .lamports()
        .checked_add(lock_lamports)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    lock_account_info.data.borrow_mut().fill(0);

    msg!("Unlocked {} tokens", amount);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::program_error::ProgramError;

    #[test]
    fn test_process_instruction_empty_data() {
        let program_id = Pubkey::new_unique();
        let accounts: Vec<AccountInfo> = vec![];
        let instruction_data: [u8; 0] = [];

        let result = process_instruction(&program_id, &accounts, &instruction_data);
        assert_eq!(
            result.unwrap_err(),
            ProgramError::Custom(LocksmithError::InvalidInstruction as u32)
        );
    }

    #[test]
    fn test_process_instruction_invalid_tag() {
        let program_id = Pubkey::new_unique();
        let accounts: Vec<AccountInfo> = vec![];
        let instruction_data = [255u8];

        let result = process_instruction(&program_id, &accounts, &instruction_data);
        assert_eq!(
            result.unwrap_err(),
            ProgramError::Custom(LocksmithError::InvalidInstruction as u32)
        );
    }

    #[test]
    fn test_lock_pda_isolation_by_lock_id() {
        let program_id = crate::id();
        let owner = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        let (pda_0, _) = Pubkey::find_program_address(
            &[LOCK_SEED, owner.as_ref(), mint.as_ref(), &0u64.to_le_bytes()],
            &program_id,
        );
        let (pda_1, _) = Pubkey::find_program_address(
            &[LOCK_SEED, owner.as_ref(), mint.as_ref(), &1u64.to_le_bytes()],
            &program_id,
        );

        assert_ne!(pda_0, pda_1);
    }

    #[test]
    fn test_lock_pda_isolation_by_owner() {
        let program_id = crate::id();
        let owner_1 = Pubkey::new_unique();
        let owner_2 = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let lock_id = 1u64.to_le_bytes();

        let (pda_1, _) = Pubkey::find_program_address(
            &[LOCK_SEED, owner_1.as_ref(), mint.as_ref(), &lock_id],
            &program_id,
        );
        let (pda_2, _) = Pubkey::find_program_address(
            &[LOCK_SEED, owner_2.as_ref(), mint.as_ref(), &lock_id],
            &program_id,
        );

        assert_ne!(pda_1, pda_2);
    }

    #[test]
    fn test_lock_pda_isolation_by_mint() {
        let program_id = crate::id();
        let owner = Pubkey::new_unique();
        let mint_1 = Pubkey::new_unique();
        let mint_2 = Pubkey::new_unique();
        let lock_id = 1u64.to_le_bytes();

        let (pda_1, _) = Pubkey::find_program_address(
            &[LOCK_SEED, owner.as_ref(), mint_1.as_ref(), &lock_id],
            &program_id,
        );
        let (pda_2, _) = Pubkey::find_program_address(
            &[LOCK_SEED, owner.as_ref(), mint_2.as_ref(), &lock_id],
            &program_id,
        );

        assert_ne!(pda_1, pda_2);
    }

    #[test]
    fn test_lock_token_pda_isolation() {
        let program_id = crate::id();
        let lock_1 = Pubkey::new_unique();
        let lock_2 = Pubkey::new_unique();

        let (token_pda_1, _) =
            Pubkey::find_program_address(&[LOCK_TOKEN_SEED, lock_1.as_ref()], &program_id);
        let (token_pda_2, _) =
            Pubkey::find_program_address(&[LOCK_TOKEN_SEED, lock_2.as_ref()], &program_id);

        assert_ne!(token_pda_1, token_pda_2);
    }

    #[test]
    fn test_usdc_mint_matches_mainnet() {
        assert_eq!(
            USDC_MINT.to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        );
    }

    #[test]
    fn test_program_id_matches_declared() {
        assert_eq!(
            crate::id().to_string(),
            "5fPbdosJd9P1gth7r9kmqc7gkgQzM2PfQHkXtQXcQyty"
        );
    }

    #[test]
    fn test_fee_usdc_is_015_usdc() {
        // 0.15 USDC with 6 decimals = 150,000
        assert_eq!(FEE_USDC, 150_000);
    }

    #[test]
    fn test_config_account_size() {
        // discriminator(8) + admin(32) + bump(1) = 41
        assert_eq!(ConfigAccount::SIZE, 41);
    }

    #[test]
    fn test_lock_account_size() {
        // discriminator(8) + owner(32) + mint(32) + amount(8) + unlock_timestamp(8)
        // + created_at(8) + lock_id(8) + bump(1) = 105
        assert_eq!(LockAccount::SIZE, 105);
    }

    #[test]
    fn test_max_lock_duration_is_10_years() {
        // 10 years in seconds = 10 * 365 * 24 * 60 * 60
        let expected_seconds: i64 = 10 * 365 * 24 * 60 * 60;
        assert_eq!(MAX_LOCK_DURATION_SECONDS, expected_seconds);
        // Verify it's approximately 315,360,000 seconds (10 years without leap years)
        assert_eq!(MAX_LOCK_DURATION_SECONDS, 315_360_000);
    }

    #[test]
    fn test_max_lock_duration_arithmetic_safety() {
        // Ensure adding MAX_LOCK_DURATION_SECONDS to a reasonable timestamp doesn't overflow
        let current_timestamp: i64 = 1_700_000_000; // ~2023
        let result = current_timestamp.checked_add(MAX_LOCK_DURATION_SECONDS);
        assert!(result.is_some());
        // Result should be around 2033
        assert!(result.unwrap() > current_timestamp);
    }

    #[test]
    fn test_system_program_check_id_validates_correctly() {
        // Test that the system program ID is recognized
        let system_program_id = solana_system_interface::program::id();
        assert!(solana_system_interface::program::check_id(&system_program_id));

        // Test that a random key is not recognized as system program
        let random_key = Pubkey::new_unique();
        assert!(!solana_system_interface::program::check_id(&random_key));
    }

    #[test]
    fn test_lock_duration_exceeded_error_code() {
        // Ensure the new error code is correct
        assert_eq!(LocksmithError::LockDurationExceeded as u32, 11);
    }
}
