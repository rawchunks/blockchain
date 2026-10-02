use pinocchio::{
    entrypoint,
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use rawchunks_common::RawchunksError;

pub mod instructions;
pub mod state;

use instructions::*;

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let (discriminator, data) = instruction_data.split_first().unwrap();

    match discriminator {
        0 => process_initialize(program_id, accounts, data),
        1 => process_mint_song(program_id, accounts, data),
        2 => process_mint_remix(program_id, accounts, data),
        3 => process_consume_license(program_id, accounts, data),
        4 => process_update_pricing(program_id, accounts, data),
        5 => process_update_metadata_uri(program_id, accounts, data),
        6 => process_authorize_consumer(program_id, accounts, data),
        7 => process_revoke_consumer(program_id, accounts, data),
        8 => process_transfer_admin(program_id, accounts, data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
