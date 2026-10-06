use {
    crate::helpers::*,
    mollusk_svm::{program::keyed_account_for_system_program, Mollusk},
    rawchunks_common::{
        state::{AuthorizedConsumer, FactoryConfig, LicenseRecord, SongAccount},
        MAX_METADATA_URI_LEN,
    },
    solana_account::Account,
    solana_address::Address,
    solana_instruction::{account_meta::AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_system_interface::program::ID as SYSTEM_PROGRAM_ID,
};

fn build_init_factory_ix(admin: &Address, config: &Address) -> Instruction {
    Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(*config, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: vec![0u8],
    }
}

fn build_mint_song_ix(
    creator: &Address,
    config: &Address,
    song_pda: &Address,
    license_amount: u64,
    license_price: u64,
    stream_price: u64,
    remixable: u8,
    uri: &str,
) -> Instruction {
    let mut data = vec![1u8];
    data.extend_from_slice(&license_amount.to_le_bytes());
    data.extend_from_slice(&license_price.to_le_bytes());
    data.extend_from_slice(&stream_price.to_le_bytes());
    data.push(remixable);
    let uri_bytes = uri.as_bytes();
    data.extend_from_slice(&(uri_bytes.len() as u16).to_le_bytes());
    data.extend_from_slice(uri_bytes);

    Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*creator, true),
            AccountMeta::new(*config, false),
            AccountMeta::new(*song_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

fn build_mint_remix_ix(
    caller: &Address,
    config: &Address,
    parent_song_pda: &Address,
    remix_song_pda: &Address,
    consumer_account: &Address,
    license_amount: u64,
    license_price: u64,
    stream_price: u64,
    parent_song_id: u64,
    remixer: &Address,
    uri: &str,
) -> Instruction {
    let mut data = vec![2u8];
    data.extend_from_slice(&license_amount.to_le_bytes());
    data.extend_from_slice(&license_price.to_le_bytes());
    data.extend_from_slice(&stream_price.to_le_bytes());
    data.extend_from_slice(&parent_song_id.to_le_bytes());
    data.extend_from_slice(remixer.as_ref());
    let uri_bytes = uri.as_bytes();
    data.extend_from_slice(&(uri_bytes.len() as u16).to_le_bytes());
    data.extend_from_slice(uri_bytes);

    Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*caller, true),
            AccountMeta::new(*config, false),
            AccountMeta::new_readonly(*parent_song_pda, false),
            AccountMeta::new(*remix_song_pda, false),
            AccountMeta::new_readonly(*consumer_account, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

fn build_consume_license_ix(
    caller: &Address,
    config: &Address,
    song_pda: &Address,
    license_record_pda: &Address,
    consumer_account: &Address,
    song_id: u64,
    licensee: &Address,
) -> Instruction {
    let mut data = vec![3u8];
    data.extend_from_slice(&song_id.to_le_bytes());
    data.extend_from_slice(licensee.as_ref());

    Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*caller, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(*song_pda, false),
            AccountMeta::new(*license_record_pda, false),
            AccountMeta::new_readonly(*consumer_account, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

fn build_update_pricing_ix(
    creator: &Address,
    song_pda: &Address,
    song_id: u64,
    new_license_price: u64,
    new_stream_price: u64,
) -> Instruction {
    let mut data = vec![4u8];
    data.extend_from_slice(&song_id.to_le_bytes());
    data.extend_from_slice(&new_license_price.to_le_bytes());
    data.extend_from_slice(&new_stream_price.to_le_bytes());

    Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*creator, true),
            AccountMeta::new(*song_pda, false),
        ],
        data,
    }
}

fn build_update_metadata_uri_ix(
    creator: &Address,
    song_pda: &Address,
    song_id: u64,
    new_uri: &str,
) -> Instruction {
    let mut data = vec![5u8];
    data.extend_from_slice(&song_id.to_le_bytes());
    let uri_bytes = new_uri.as_bytes();
    data.extend_from_slice(&(uri_bytes.len() as u16).to_le_bytes());
    data.extend_from_slice(uri_bytes);

    Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*creator, true),
            AccountMeta::new(*song_pda, false),
        ],
        data,
    }
}

fn build_authorize_consumer_ix(
    admin: &Address,
    config: &Address,
    consumer_pda: &Address,
    consumer: &Address,
) -> Instruction {
    let mut data = vec![6u8];
    data.extend_from_slice(consumer.as_ref());

    Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(*consumer_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

fn build_revoke_consumer_ix(
    admin: &Address,
    config: &Address,
    consumer_pda: &Address,
    consumer: &Address,
) -> Instruction {
    let mut data = vec![7u8];
    data.extend_from_slice(consumer.as_ref());

    Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(*consumer_pda, false),
        ],
        data,
    }
}

fn build_transfer_admin_ix(admin: &Address, config: &Address, new_admin: &Address) -> Instruction {
    let mut data = vec![8u8];
    data.extend_from_slice(new_admin.as_ref());

    Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*admin, true),
            AccountMeta::new(*config, false),
        ],
        data,
    }
}

#[test]
fn test_factory_initialize_and_state_verification() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, bump) = find_factory_config_pda(&FACTORY_PROGRAM_ID);

    let ix = build_init_factory_ix(&admin.pubkey(), &config_pda);
    let cu = env.send_tx(ix, &[&admin]).expect("init factory");
    assert!(cu < 15_000, "Compute budget exceeded: {}", cu);

    let account = env.svm.get_account(&config_pda).expect("config exists");
    assert_eq!(account.data.len(), FactoryConfig::LEN);
    assert_eq!(account.data[0], FactoryConfig::DISCRIMINATOR);
    assert_eq!(account.data[1], bump);
    assert_eq!(&account.data[8..40], admin.pubkey().as_ref());

    let next_song_id = u64::from_le_bytes(account.data[40..48].try_into().unwrap());
    let total_songs = u64::from_le_bytes(account.data[48..56].try_into().unwrap());
    assert_eq!(next_song_id, 1);
    assert_eq!(total_songs, 0);
}

#[test]
fn test_factory_duplicate_init_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);

    let ix1 = build_init_factory_ix(&admin.pubkey(), &config_pda);
    env.send_tx(ix1, &[&admin]).unwrap();

    let ix2 = build_init_factory_ix(&admin.pubkey(), &config_pda);
    let res = env.send_tx(ix2, &[&admin]);
    assert!(res.is_err());
}

#[test]
fn test_factory_init_invalid_seeds_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let invalid_config = Keypair::new().pubkey();

    let ix = build_init_factory_ix(&admin.pubkey(), &invalid_config);
    let res = env.send_tx(ix, &[&admin]);
    assert!(res.is_err());
}

#[test]
fn test_factory_init_missing_signature_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let payer = env.payer();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);

    let mut ix = build_init_factory_ix(&admin.pubkey(), &config_pda);
    ix.accounts[0].is_signer = false;
    let res = env.send_tx(ix, &[&payer]);
    assert!(res.is_err());
}
