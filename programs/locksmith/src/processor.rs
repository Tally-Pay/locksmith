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
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::state::Account as TokenAccount;

use crate::error::LocksmithError;
use crate::instruction::LocksmithInstruction;
use crate::state::{
    ConfigAccount, LockAccount, CONFIG_SEED, FEE_USDC, FEE_VAULT_SEED, LOCK_SEED, LOCK_TOKEN_SEED,
    USDC_MINT,
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
    let _token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
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
    let _config_info = next_account_info(account_info_iter)?;
    let fee_vault_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if amount == 0 {
        return Err(LocksmithError::InvalidAmount.into());
    }

    let clock = Clock::get()?;
    if unlock_timestamp <= clock.unix_timestamp {
        return Err(LocksmithError::InvalidTimestamp.into());
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
