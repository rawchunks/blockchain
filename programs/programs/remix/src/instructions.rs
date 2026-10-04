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

fn original(
    program_id: &Address, remix_config_account: &AccountView,
    remix_authority: &AccountView, factory_program: &AccountView,
    factory_config: &AccountView, original_song: &AccountView, original_song_id: u64,
) -> Result<(u8, u64, [u8; 32], u64), ProgramError> {
    let (expected_remix_pda, _) = RemixConfig::find_pda(program_id);
    if remix_config_account.address() != &expected_remix_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let (expected_auth_pda, authority_bump) = RemixConfig::find_authority_pda(program_id);
    if remix_authority.address() != &expected_auth_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let configured_factory = {
        let config = RemixConfig::from_account_info_readonly(remix_config_account)?;
        config.factory_program
    };

    if factory_program.address().as_ref() != &configured_factory {
        return Err(RawchunksError::InvalidAddress.into());
    }

    let (expected_factory_cfg, _) = FactoryConfig::find_pda(factory_program.address());
    if factory_config.address() != &expected_factory_cfg {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    let (expected_orig_song, _) = SongAccount::find_pda(original_song_id, factory_program.address());
    if original_song.address() != &expected_orig_song {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    if !remix_config_account.owned_by(program_id)
        || !factory_config.owned_by(factory_program.address())
        || !original_song.owned_by(factory_program.address()) {
        return Err(RawchunksError::InvalidAccountOwner.into());
    }
    let orig_song = SongAccount::from_account_info_readonly(original_song)?;
    if orig_song.song_id != original_song_id {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    if orig_song.remixable == 0 {
        return Err(RawchunksError::SongNotRemixable.into());
    }
    if orig_song.is_remix == 1 {
        return Err(RawchunksError::RemixCannotBeRemixed.into());
    }
    Ok((authority_bump, orig_song.license_price, orig_song.creator, orig_song.license_amount))
}

fn licensed(
    factory_program: &Address, account: &AccountView, song_id: u64, user: &Address,
) -> Result<bool, ProgramError> {
    let user_bytes: [u8; 32] = user.as_ref().try_into().unwrap();
    let (expected, _) = LicenseRecord::find_pda(song_id, &user_bytes, factory_program);
    if account.address() != &expected {
        return Err(RawchunksError::InvalidSeeds.into());
    }
    if account.data_len() == 0 { return Ok(false); }
    if !account.owned_by(factory_program) {
        return Err(RawchunksError::InvalidAccountOwner.into());
    }
    let record = LicenseRecord::from_account_info_readonly(account)?;
    if record.song_id != song_id || record.user != user_bytes {
        return Err(RawchunksError::InvalidSeeds.into());
    }
    Ok(true)
}

pub fn process_buy_license(
    program_id: &Address, accounts: &[AccountView], data: &[u8],
) -> ProgramResult {
    let [buyer, remix_authority, remix_config_account, factory_program, royalties_program,
        factory_config, original_song, license_record, remix_consumer_account,
        payer_token, creator_token, treasury_token, royalties_config, token_program,
        system_program, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    if !buyer.is_signer() { return Err(RawchunksError::MissingRequiredSignature.into()); }
    if data.len() != 8 { return Err(RawchunksError::InvalidInstructionData.into()); }
    let song_id = u64::from_le_bytes(data.try_into().unwrap());
    let (authority_bump, price, creator, remaining) = original(program_id,
        remix_config_account, remix_authority, factory_program, factory_config, original_song, song_id)?;
    let config = RemixConfig::from_account_info_readonly(remix_config_account)?;
    if royalties_program.address().as_ref() != &config.royalties_program {
        return Err(RawchunksError::InvalidAddress.into());
    }
    if licensed(factory_program.address(), license_record, song_id, buyer.address())? { return Ok(()); }
    if remaining == 0 { return Err(RawchunksError::NoLicensesAvailable.into()); }
    // Step 1: Distribute USDC license payment via royalties CPI if price > 0
    if price > 0 {
        let (expected_roy_pda, _) = RoyaltiesConfig::find_pda(royalties_program.address());
        if royalties_config.address() != &expected_roy_pda {
            return Err(RawchunksError::InvalidSeeds.into());
        }

        // Verify creator_token is owned by the authentic original song creator
        let creator_owner = unsafe {
            let acc = PinocchioTokenAccount::from_account_view_unchecked(creator_token)?;
            *acc.owner()
        };
        if creator_owner.as_ref() != &creator {
            return Err(RawchunksError::InvalidAddress.into());
        }

        let mut royalties_ix_data = [0u8; 49];
        royalties_ix_data[0] = 1; // distribute_license_payment discriminator
        royalties_ix_data[1..9].copy_from_slice(&song_id.to_le_bytes());
        royalties_ix_data[9..17].copy_from_slice(&price.to_le_bytes());
        royalties_ix_data[17..49].copy_from_slice(&creator);

        let buyer_addr = *buyer.address();
        let payer_token_addr = *payer_token.address();
        let creator_token_addr = *creator_token.address();
        let treasury_token_addr = *treasury_token.address();
        let royalties_config_addr = *royalties_config.address();
        let token_program_addr = *token_program.address();

        let royalties_accounts = [
            InstructionAccount::writable_signer(&buyer_addr),
            InstructionAccount::writable(&payer_token_addr),
            InstructionAccount::writable(&creator_token_addr),
            InstructionAccount::writable(&treasury_token_addr),
            InstructionAccount::readonly(&royalties_config_addr),
            InstructionAccount::readonly(&token_program_addr),
        ];

        let royalties_ix = InstructionView {
            program_id: royalties_program.address(),
            accounts: &royalties_accounts,
            data: &royalties_ix_data,
        };

        invoke(&royalties_ix, &[
            buyer,
            payer_token,
            creator_token,
            treasury_token,
            royalties_config,
            token_program,
        ])?;
    }


    let needed = Rent::get()?.try_minimum_balance(LicenseRecord::LEN)?;
    if remix_authority.lamports() < needed {
        SystemTransfer { from: buyer, to: remix_authority,
            lamports: needed - remix_authority.lamports() }.invoke()?;
    }
    // Prepare authority signer for Factory CPIs
    let auth_bump_bytes = [authority_bump];
    let auth_seeds = [
        Seed::from(SEED_REMIX_AUTHORITY),
        Seed::from(&auth_bump_bytes),
    ];
    let auth_signer = Signer::from(&auth_seeds);

    let remix_auth_addr = *remix_authority.address();
    let factory_cfg_addr = *factory_config.address();
    let original_song_addr = *original_song.address();
    let license_rec_addr = *license_record.address();
    let remix_consumer_addr = *remix_consumer_account.address();
    let system_prog_addr = *system_program.address();

    // Step 2: Consume license via factory CPI
    let mut consume_ix_data = [0u8; 41];
    consume_ix_data[0] = 3;
    consume_ix_data[1..9].copy_from_slice(&song_id.to_le_bytes());
    consume_ix_data[9..41].copy_from_slice(buyer.address().as_ref());

    let consume_accounts = [
        InstructionAccount::writable_signer(&remix_auth_addr),
        InstructionAccount::readonly(&factory_cfg_addr),
        InstructionAccount::writable(&original_song_addr),
        InstructionAccount::writable(&license_rec_addr),
        InstructionAccount::readonly(&remix_consumer_addr),
        InstructionAccount::readonly(&system_prog_addr),
    ];

    let consume_ix = InstructionView {
        program_id: factory_program.address(),
        accounts: &consume_accounts,
        data: &consume_ix_data,
    };

    invoke_signed(&consume_ix, &[
        remix_authority,
        factory_config,
        original_song,
        license_record,
        remix_consumer_account,
        system_program,
    ], &[auth_signer.clone()])?;


    log!("LicensePurchased: song_id={}, price={}", song_id, price);
    Ok(())
}
