use pinocchio::{
    cpi::{invoke, invoke_signed, Seed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_log::log;
use pinocchio_system::instructions::{CreateAccount, Transfer as SystemTransfer};
use pinocchio_token::state::Account as PinocchioTokenAccount;
use rawchunks_common::{
    FactoryConfig, LicenseRecord, RawchunksError, RoyaltiesConfig, SongAccount,
    MAX_METADATA_URI_LEN, SEED_REMIX, SEED_REMIX_AUTHORITY, SEED_REMIX_COUNT,
};

use crate::state::{RemixConfig, RemixRecord, SongRemixCount};

pub fn process_initialize(
    program_id: &Address,
    accounts: &[AccountView],
    data: &[u8],
) -> ProgramResult {
    if accounts.len() < 3 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let admin = &accounts[0];
    let config_account = &accounts[1];

    if !admin.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    if data.len() < 64 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let factory_program: [u8; 32] = data[0..32].try_into().unwrap();
    let royalties_program: [u8; 32] = data[32..64].try_into().unwrap();

    let admin_address: [u8; 32] = if data.len() >= 96 {
        data[64..96].try_into().unwrap()
    } else {
        let mut addr = [0u8; 32];
        addr.copy_from_slice(admin.address().as_ref());
        addr
    };

    let (expected_pda, bump) = RemixConfig::find_pda(program_id);
    if config_account.address() != &expected_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    if config_account.data_len() > 0 {
        return Err(RawchunksError::AlreadyInitialized.into());
    }

    let (_, authority_bump) = RemixConfig::find_authority_pda(program_id);

    let bump_bytes = [bump];
    let seeds = [
        Seed::from(SEED_REMIX),
        Seed::from(&bump_bytes),
    ];
    let signer = Signer::from(&seeds);

    CreateAccount::with_minimum_balance(
        admin,
        config_account,
        RemixConfig::LEN as u64,
        program_id,
        None,
    )?
    .invoke_signed(&[signer])?;

    let config = RemixConfig::from_account_info_init(config_account)?;
    config.discriminator = RemixConfig::DISCRIMINATOR;
    config.bump = bump;
    config.authority_bump = authority_bump;
    config._padding = [0u8; 5];
    config.total_remixes = 0;
    config.admin = admin_address;
    config.factory_program = factory_program;
    config.royalties_program = royalties_program;

    log!("Remix program initialized successfully");

    Ok(())
}
