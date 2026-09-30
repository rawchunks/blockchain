use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use pinocchio_log::log;
use pinocchio_system::instructions::CreateAccount;
use rawchunks_common::{
    RawchunksError, MAX_METADATA_URI_LEN, SEED_CONSUMER, SEED_FACTORY, SEED_LICENSE, SEED_SONG,
};

use crate::state::{AuthorizedConsumer, FactoryConfig, LicenseRecord, SongAccount};

fn check_is_authorized_consumer(
    caller: &AccountView,
    admin_address: &[u8; 32],
    consumer_account: Option<&AccountView>,
    program_id: &Address,
) -> Result<(), ProgramError> {
    if caller.address().as_ref() == admin_address {
        return Ok(());
    }
    let consumer_acc = consumer_account.ok_or(RawchunksError::NotAuthorized)?;
    let (expected_pda, _) = AuthorizedConsumer::find_pda(
        caller.address().as_ref().try_into().unwrap(),
        program_id,
    );
    if consumer_acc.address() != &expected_pda {
        return Err(RawchunksError::NotAuthorized.into());
    }
    let consumer = AuthorizedConsumer::from_account_info_readonly(consumer_acc)?;
    if consumer.consumer.as_ref() != caller.address().as_ref() {
        return Err(RawchunksError::NotAuthorized.into());
    }
    Ok(())
}

pub fn process_initialize(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let [admin, config_account, _system_program, _rem @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !admin.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    let admin_address: [u8; 32] = if data.len() >= 32 {
        data[0..32].try_into().map_err(|_| RawchunksError::InvalidInstructionData)?
    } else {
        let mut addr = [0u8; 32];
        addr.copy_from_slice(admin.address().as_ref());
        addr
    };

    let (expected_pda, bump) = FactoryConfig::find_pda(program_id);
    if config_account.address() != &expected_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    if config_account.data_len() > 0 {
        return Err(RawchunksError::AlreadyInitialized.into());
    }

    let bump_bytes = [bump];
    let seeds = [
        Seed::from(SEED_FACTORY),
        Seed::from(&bump_bytes),
    ];
    let signer = Signer::from(&seeds);

    CreateAccount::with_minimum_balance(
        admin,
        config_account,
        FactoryConfig::LEN as u64,
        program_id,
        None,
    )?
    .invoke_signed(&[signer])?;

    let config = FactoryConfig::from_account_info_init(config_account)?;
    config.discriminator = FactoryConfig::DISCRIMINATOR;
    config.bump = bump;
    config._padding = [0u8; 6];
    config.admin = admin_address;
    config.next_song_id = 1;
    config.total_songs = 0;

    log!("Factory initialized successfully");

    Ok(())
}
