pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint {
    use solana_program::{
        account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
    };

    use crate::processor::process_instruction;

    entrypoint!(program_entrypoint);

    fn program_entrypoint(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        process_instruction(program_id, accounts, instruction_data)
    }
}

solana_program::declare_id!("5fPbdosJd9P1gth7r9kmqc7gkgQzM2PfQHkXtQXcQyty");
