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

pub fn process_mint_song(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let [creator, config_account, song_account, _system_program, _rem @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !creator.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    // Minimum data: license_amount(8) + license_price(8) + stream_price(8) + remixable(1) + uri_len(2) = 27
    if data.len() < 27 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let license_amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let license_price = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let stream_price = u64::from_le_bytes(data[16..24].try_into().unwrap());
    let remixable = data[24];
    let uri_len = u16::from_le_bytes(data[25..27].try_into().unwrap()) as usize;

    if uri_len == 0 {
        return Err(RawchunksError::EmptyMetadataUri.into());
    }
    if uri_len > MAX_METADATA_URI_LEN || data.len() < 27 + uri_len {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let uri_bytes = &data[27..27 + uri_len];

    let (expected_config_pda, _) = FactoryConfig::find_pda(program_id);
    if config_account.address() != &expected_config_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let song_id = {
        let config = FactoryConfig::from_account_info(config_account)?;
        let id = config.next_song_id;
        config.next_song_id = id.checked_add(1).ok_or(RawchunksError::MathOverflow)?;
        config.total_songs = config.total_songs.checked_add(1).ok_or(RawchunksError::MathOverflow)?;
        id
    };

    let (expected_song_pda, song_bump) = SongAccount::find_pda(song_id, program_id);
    if song_account.address() != &expected_song_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let song_id_bytes = song_id.to_le_bytes();
    let song_bump_bytes = [song_bump];
    let seeds = [
        Seed::from(SEED_SONG),
        Seed::from(&song_id_bytes),
        Seed::from(&song_bump_bytes),
    ];
    let signer = Signer::from(&seeds);

    CreateAccount::with_minimum_balance(
        creator,
        song_account,
        SongAccount::LEN as u64,
        program_id,
        None,
    )?
    .invoke_signed(&[signer])?;

    let mut creator_addr = [0u8; 32];
    creator_addr.copy_from_slice(creator.address().as_ref());

    let mut metadata_uri = [0u8; MAX_METADATA_URI_LEN];
    metadata_uri[..uri_len].copy_from_slice(uri_bytes);

    let song = SongAccount::from_account_info_init(song_account)?;
    song.discriminator = SongAccount::DISCRIMINATOR;
    song.bump = song_bump;
    song.remixable = remixable;
    song.is_remix = 0;
    song.uri_len = uri_len as u16;
    song._padding = [0u8; 2];
    song.song_id = song_id;
    song.parent_song_id = 0;
    song.license_amount = license_amount;
    song.license_price = license_price;
    song.stream_price = stream_price;
    song.creator = creator_addr;
    song.metadata_uri = metadata_uri;

    log!(
        "SongMinted: song_id={}, license_amount={}, license_price={}",
        song_id,
        license_amount,
        license_price
    );

    Ok(())
}

pub fn process_mint_remix(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let [caller, config_account, parent_song_account, remix_song_account, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !caller.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    // Minimum data: license_amount(8) + license_price(8) + stream_price(8) + parent_song_id(8) + remixer(32) + uri_len(2) = 66
    if data.len() < 66 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let license_amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let license_price = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let stream_price = u64::from_le_bytes(data[16..24].try_into().unwrap());
    let parent_song_id = u64::from_le_bytes(data[24..32].try_into().unwrap());
    let remixer: [u8; 32] = data[32..64].try_into().unwrap();
    let uri_len = u16::from_le_bytes(data[64..66].try_into().unwrap()) as usize;

    if uri_len == 0 {
        return Err(RawchunksError::EmptyMetadataUri.into());
    }
    if uri_len > MAX_METADATA_URI_LEN || data.len() < 66 + uri_len {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let uri_bytes = &data[66..66 + uri_len];

    let (expected_config_pda, _) = FactoryConfig::find_pda(program_id);
    if config_account.address() != &expected_config_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let consumer_account = rest.first();
    let config_admin = {
        let config = FactoryConfig::from_account_info_readonly(config_account)?;
        config.admin
    };
    check_is_authorized_consumer(caller, &config_admin, consumer_account, program_id)?;

    // Check parent song
    let (expected_parent_pda, _) = SongAccount::find_pda(parent_song_id, program_id);
    if parent_song_account.address() != &expected_parent_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }
    let parent_song = SongAccount::from_account_info_readonly(parent_song_account)?;
    if parent_song.remixable == 0 {
        return Err(RawchunksError::SongNotRemixable.into());
    }
    if parent_song.is_remix == 1 {
        return Err(RawchunksError::RemixCannotBeRemixed.into());
    }

    let song_id = {
        let config = FactoryConfig::from_account_info(config_account)?;
        let id = config.next_song_id;
        config.next_song_id = id.checked_add(1).ok_or(RawchunksError::MathOverflow)?;
        config.total_songs = config.total_songs.checked_add(1).ok_or(RawchunksError::MathOverflow)?;
        id
    };

    let (expected_song_pda, song_bump) = SongAccount::find_pda(song_id, program_id);
    if remix_song_account.address() != &expected_song_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let song_id_bytes = song_id.to_le_bytes();
    let song_bump_bytes = [song_bump];
    let seeds = [
        Seed::from(SEED_SONG),
        Seed::from(&song_id_bytes),
        Seed::from(&song_bump_bytes),
    ];
    let signer = Signer::from(&seeds);

    CreateAccount::with_minimum_balance(
        caller,
        remix_song_account,
        SongAccount::LEN as u64,
        program_id,
        None,
    )?
    .invoke_signed(&[signer])?;

    let mut metadata_uri = [0u8; MAX_METADATA_URI_LEN];
    metadata_uri[..uri_len].copy_from_slice(uri_bytes);

    let remix_song = SongAccount::from_account_info_init(remix_song_account)?;
    remix_song.discriminator = SongAccount::DISCRIMINATOR;
    remix_song.bump = song_bump;
    remix_song.remixable = 0; // Remixed songs are not remixable
    remix_song.is_remix = 1;
    remix_song.uri_len = uri_len as u16;
    remix_song._padding = [0u8; 2];
    remix_song.song_id = song_id;
    remix_song.parent_song_id = parent_song_id;
    remix_song.license_amount = license_amount;
    remix_song.license_price = license_price;
    remix_song.stream_price = stream_price;
    remix_song.creator = remixer;
    remix_song.metadata_uri = metadata_uri;

    log!(
        "RemixMinted: song_id={}, parent_song_id={}",
        song_id,
        parent_song_id
    );

    Ok(())
}

pub fn process_consume_license(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let [caller, config_account, song_account, license_record_account, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !caller.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    if data.len() < 40 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let song_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let licensee: [u8; 32] = data[8..40].try_into().unwrap();

    let (expected_config_pda, _) = FactoryConfig::find_pda(program_id);
    if config_account.address() != &expected_config_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let consumer_account = rest.first();
    let config_admin = {
        let config = FactoryConfig::from_account_info_readonly(config_account)?;
        config.admin
    };
    check_is_authorized_consumer(caller, &config_admin, consumer_account, program_id)?;

    let (expected_song_pda, _) = SongAccount::find_pda(song_id, program_id);
    if song_account.address() != &expected_song_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let song = SongAccount::from_account_info(song_account)?;
    if song.remixable == 0 {
        return Err(RawchunksError::SongNotRemixable.into());
    }
    if song.is_remix == 1 {
        return Err(RawchunksError::RemixCannotBeRemixed.into());
    }
    if song.license_amount == 0 {
        return Err(RawchunksError::NoLicensesAvailable.into());
    }

    song.license_amount -= 1;
    let licenses_left = song.license_amount;

    // Check or create license_record_account PDA
    let (expected_license_pda, license_bump) = LicenseRecord::find_pda(song_id, &licensee, program_id);
    if license_record_account.address() != &expected_license_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    if license_record_account.data_len() == 0 {
        let song_id_bytes = song_id.to_le_bytes();
        let license_bump_bytes = [license_bump];
        let seeds = [
            Seed::from(SEED_LICENSE),
            Seed::from(&song_id_bytes),
            Seed::from(licensee.as_ref()),
            Seed::from(&license_bump_bytes),
        ];
        let signer = Signer::from(&seeds);

        CreateAccount::with_minimum_balance(
            caller,
            license_record_account,
            LicenseRecord::LEN as u64,
            program_id,
            None,
        )?
        .invoke_signed(&[signer])?;

        let record = LicenseRecord::from_account_info_init(license_record_account)?;
        record.discriminator = LicenseRecord::DISCRIMINATOR;
        record.bump = license_bump;
        record._padding = [0u8; 6];
        record.song_id = song_id;
        record.user = licensee;
    }

    log!(
        "LicenseConsumed: song_id={}, licenses_left={}",
        song_id,
        licenses_left
    );

    Ok(())
}

pub fn process_update_pricing(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let [creator, song_account, _rem @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !creator.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    if data.len() < 24 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let song_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let new_license_price = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let new_stream_price = u64::from_le_bytes(data[16..24].try_into().unwrap());

    let (expected_song_pda, _) = SongAccount::find_pda(song_id, program_id);
    if song_account.address() != &expected_song_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let song = SongAccount::from_account_info(song_account)?;
    if song.creator.as_ref() != creator.address().as_ref() {
        return Err(RawchunksError::NotSongCreator.into());
    }

    song.license_price = new_license_price;
    song.stream_price = new_stream_price;

    log!("Song pricing updated: song_id={}", song_id);

    Ok(())
}

pub fn process_update_metadata_uri(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let [creator, song_account, _rem @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !creator.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    if data.len() < 10 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let song_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let uri_len = u16::from_le_bytes(data[8..10].try_into().unwrap()) as usize;

    if uri_len == 0 {
        return Err(RawchunksError::EmptyMetadataUri.into());
    }
    if uri_len > MAX_METADATA_URI_LEN || data.len() < 10 + uri_len {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let uri_bytes = &data[10..10 + uri_len];

    let (expected_song_pda, _) = SongAccount::find_pda(song_id, program_id);
    if song_account.address() != &expected_song_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let song = SongAccount::from_account_info(song_account)?;
    if song.creator.as_ref() != creator.address().as_ref() {
        return Err(RawchunksError::NotSongCreator.into());
    }

    let mut metadata_uri = [0u8; MAX_METADATA_URI_LEN];
    metadata_uri[..uri_len].copy_from_slice(uri_bytes);

    song.uri_len = uri_len as u16;
    song.metadata_uri = metadata_uri;

    log!("Song metadata URI updated: song_id={}", song_id);

    Ok(())
}
