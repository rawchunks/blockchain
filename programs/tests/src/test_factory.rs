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

#[test]
fn test_factory_mint_standard_remixable_song() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 2_000_000_000);

    let song_id = 1u64;
    let (song_pda, bump) = find_song_pda(song_id, &FACTORY_PROGRAM_ID);
    let uri = "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";

    let mint_ix = build_mint_song_ix(
        &creator.pubkey(),
        &config_pda,
        &song_pda,
        10,
        5_000_000,
        10_000,
        1,
        uri,
    );
    let cu = env.send_tx(mint_ix, &[&creator]).expect("mint song");
    assert!(cu < 15_000, "Compute budget exceeded: {}", cu);

    let account = env.svm.get_account(&song_pda).expect("song exists");
    assert_eq!(account.data.len(), SongAccount::LEN);
    assert_eq!(account.data[0], SongAccount::DISCRIMINATOR);
    assert_eq!(account.data[1], bump);
    assert_eq!(account.data[2], 1); // remixable
    assert_eq!(account.data[3], 0); // is_remix
    let parsed_uri_len = u16::from_le_bytes(account.data[4..6].try_into().unwrap()) as usize;
    assert_eq!(parsed_uri_len, uri.len());

    let parsed_song_id = u64::from_le_bytes(account.data[8..16].try_into().unwrap());
    assert_eq!(parsed_song_id, 1);
    let parsed_parent_id = u64::from_le_bytes(account.data[16..24].try_into().unwrap());
    assert_eq!(parsed_parent_id, 0);
    let parsed_licenses = u64::from_le_bytes(account.data[24..32].try_into().unwrap());
    assert_eq!(parsed_licenses, 10);
    let parsed_lic_price = u64::from_le_bytes(account.data[32..40].try_into().unwrap());
    assert_eq!(parsed_lic_price, 5_000_000);
    let parsed_str_price = u64::from_le_bytes(account.data[40..48].try_into().unwrap());
    assert_eq!(parsed_str_price, 10_000);
    assert_eq!(&account.data[48..80], creator.pubkey().as_ref());
    assert_eq!(&account.data[80..80 + uri.len()], uri.as_bytes());

    let cfg_acc = env.svm.get_account(&config_pda).unwrap();
    let next_id = u64::from_le_bytes(cfg_acc.data[40..48].try_into().unwrap());
    let total = u64::from_le_bytes(cfg_acc.data[48..56].try_into().unwrap());
    assert_eq!(next_id, 2);
    assert_eq!(total, 1);
}

#[test]
fn test_factory_mint_unlimited_licenses_and_free_pricing() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 2_000_000_000);

    let song_id = 1u64;
    let (song_pda, _) = find_song_pda(song_id, &FACTORY_PROGRAM_ID);
    let uri = "ipfs://unlimited-free-track";

    let mint_ix = build_mint_song_ix(
        &creator.pubkey(),
        &config_pda,
        &song_pda,
        u64::MAX,
        0,
        0,
        1,
        uri,
    );
    env.send_tx(mint_ix, &[&creator]).unwrap();

    let account = env.svm.get_account(&song_pda).unwrap();
    let parsed_licenses = u64::from_le_bytes(account.data[24..32].try_into().unwrap());
    let parsed_price = u64::from_le_bytes(account.data[32..40].try_into().unwrap());
    assert_eq!(parsed_licenses, u64::MAX);
    assert_eq!(parsed_price, 0);
}

#[test]
fn test_factory_mint_non_remixable_song() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 2_000_000_000);

    let song_id = 1u64;
    let (song_pda, _) = find_song_pda(song_id, &FACTORY_PROGRAM_ID);
    let uri = "ipfs://exclusive-copyright-no-remix";

    let mint_ix = build_mint_song_ix(
        &creator.pubkey(),
        &config_pda,
        &song_pda,
        0,
        0,
        50_000,
        0, // remixable = 0
        uri,
    );
    env.send_tx(mint_ix, &[&creator]).unwrap();

    let account = env.svm.get_account(&song_pda).unwrap();
    assert_eq!(account.data[2], 0); // remixable = 0
}

#[test]
fn test_factory_mint_sequential_songs() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 5_000_000_000);

    for expected_id in 1..=3u64 {
        let (song_pda, _) = find_song_pda(expected_id, &FACTORY_PROGRAM_ID);
        let uri = format!("ipfs://song-sequential-{}", expected_id);
        let mint_ix = build_mint_song_ix(
            &creator.pubkey(),
            &config_pda,
            &song_pda,
            5,
            1_000_000,
            5_000,
            1,
            &uri,
        );
        env.send_tx(mint_ix, &[&creator]).unwrap();

        let song_acc = env.svm.get_account(&song_pda).unwrap();
        let id = u64::from_le_bytes(song_acc.data[8..16].try_into().unwrap());
        assert_eq!(id, expected_id);

        let cfg_acc = env.svm.get_account(&config_pda).unwrap();
        let next_id = u64::from_le_bytes(cfg_acc.data[40..48].try_into().unwrap());
        let total = u64::from_le_bytes(cfg_acc.data[48..56].try_into().unwrap());
        assert_eq!(next_id, expected_id + 1);
        assert_eq!(total, expected_id);
    }
}

#[test]
fn test_factory_mint_empty_and_oversized_metadata_uri_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 2_000_000_000);
    let (song_pda, _) = find_song_pda(1, &FACTORY_PROGRAM_ID);

    let empty_uri_ix = build_mint_song_ix(&creator.pubkey(), &config_pda, &song_pda, 1, 100, 10, 1, "");
    let res = env.send_tx(empty_uri_ix, &[&creator]);
    assert!(res.is_err(), "Empty URI must be rejected");

    let oversized_uri = "a".repeat(MAX_METADATA_URI_LEN + 1);
    let oversized_uri_ix = build_mint_song_ix(&creator.pubkey(), &config_pda, &song_pda, 1, 100, 10, 1, &oversized_uri);
    let res = env.send_tx(oversized_uri_ix, &[&creator]);
    assert!(res.is_err(), "Oversized URI must be rejected");
}

#[test]
fn test_factory_consumer_authorization_and_revocation_lifecycle() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let consumer = Keypair::new().pubkey();
    let (consumer_pda, bump) = find_authorized_consumer_pda(&consumer, &FACTORY_PROGRAM_ID);

    let auth_ix = build_authorize_consumer_ix(&admin.pubkey(), &config_pda, &consumer_pda, &consumer);
    let cu = env.send_tx(auth_ix, &[&admin]).expect("authorize consumer");
    assert!(cu < 15_000);

    let account = env.svm.get_account(&consumer_pda).expect("consumer PDA created");
    assert_eq!(account.data.len(), AuthorizedConsumer::LEN);
    assert_eq!(account.data[0], AuthorizedConsumer::DISCRIMINATOR);
    assert_eq!(account.data[1], bump);
    assert_eq!(&account.data[8..40], consumer.as_ref());

    // Duplicate authorization rejected
    let dup_auth_ix = build_authorize_consumer_ix(&admin.pubkey(), &config_pda, &consumer_pda, &consumer);
    let res = env.send_tx(dup_auth_ix, &[&admin]);
    assert!(res.is_err());

    // Attacker cannot authorize consumer
    let attacker = Keypair::new();
    env.airdrop(&attacker.pubkey(), 1_000_000_000);
    let attacker_consumer = Keypair::new().pubkey();
    let (att_pda, _) = find_authorized_consumer_pda(&attacker_consumer, &FACTORY_PROGRAM_ID);
    let att_auth_ix = build_authorize_consumer_ix(&attacker.pubkey(), &config_pda, &att_pda, &attacker_consumer);
    let res = env.send_tx(att_auth_ix, &[&attacker]);
    assert!(res.is_err());

    // Attacker cannot revoke consumer
    let att_revoke_ix = build_revoke_consumer_ix(&attacker.pubkey(), &config_pda, &consumer_pda, &consumer);
    let res = env.send_tx(att_revoke_ix, &[&attacker]);
    assert!(res.is_err());

    // Admin revokes consumer: account closed and lamports returned to admin
    let admin_balance_before = env.svm.get_account(&admin.pubkey()).unwrap().lamports;
    let revoke_ix = build_revoke_consumer_ix(&admin.pubkey(), &config_pda, &consumer_pda, &consumer);
    env.send_tx(revoke_ix, &[&admin]).expect("revoke consumer");

    let consumer_acc_after = env.svm.get_account(&consumer_pda);
    assert!(consumer_acc_after.is_none() || consumer_acc_after.unwrap().lamports == 0);
    let admin_balance_after = env.svm.get_account(&admin.pubkey()).unwrap().lamports;
    assert!(admin_balance_after > admin_balance_before - 100_000);
}

#[test]
fn test_factory_consume_license_authorized_consumer_and_depletion() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let consumer = Keypair::new();
    env.airdrop(&consumer.pubkey(), 2_000_000_000);
    let (consumer_pda, _) = find_authorized_consumer_pda(&consumer.pubkey(), &FACTORY_PROGRAM_ID);
    env.send_tx(build_authorize_consumer_ix(&admin.pubkey(), &config_pda, &consumer_pda, &consumer.pubkey()), &[&admin]).unwrap();

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 2_000_000_000);
    let song_id = 1u64;
    let (song_pda, _) = find_song_pda(song_id, &FACTORY_PROGRAM_ID);
    env.send_tx(build_mint_song_ix(&creator.pubkey(), &config_pda, &song_pda, 2, 100, 10, 1, "ipfs://test-depletion"), &[&creator]).unwrap();

    let user_a = Keypair::new().pubkey();
    let (lic_a_pda, bump_a) = find_license_record_pda(song_id, &user_a, &FACTORY_PROGRAM_ID);

    // Consume 1st license (remaining: 1)
    let consume_a = build_consume_license_ix(&consumer.pubkey(), &config_pda, &song_pda, &lic_a_pda, &consumer_pda, song_id, &user_a);
    let cu = env.send_tx(consume_a, &[&consumer]).expect("consume 1");
    assert!(cu < 25_000);

    let song_acc = env.svm.get_account(&song_pda).unwrap();
    let rem1 = u64::from_le_bytes(song_acc.data[24..32].try_into().unwrap());
    assert_eq!(rem1, 1);

    let lic_a_acc = env.svm.get_account(&lic_a_pda).unwrap();
    assert_eq!(lic_a_acc.data[0], LicenseRecord::DISCRIMINATOR);
    assert_eq!(lic_a_acc.data[1], bump_a);
    let lic_a_song_id = u64::from_le_bytes(lic_a_acc.data[8..16].try_into().unwrap());
    assert_eq!(lic_a_song_id, 1);
    assert_eq!(&lic_a_acc.data[16..48], user_a.as_ref());

    // Consume 2nd license (remaining: 0)
    let user_b = Keypair::new().pubkey();
    let (lic_b_pda, _) = find_license_record_pda(song_id, &user_b, &FACTORY_PROGRAM_ID);
    let consume_b = build_consume_license_ix(&consumer.pubkey(), &config_pda, &song_pda, &lic_b_pda, &consumer_pda, song_id, &user_b);
    env.send_tx(consume_b, &[&consumer]).expect("consume 2");

    let song_acc = env.svm.get_account(&song_pda).unwrap();
    let rem2 = u64::from_le_bytes(song_acc.data[24..32].try_into().unwrap());
    assert_eq!(rem2, 0);

    // Consume 3rd license -> Depleted! Must fail with NoLicensesAvailable
    let user_c = Keypair::new().pubkey();
    let (lic_c_pda, _) = find_license_record_pda(song_id, &user_c, &FACTORY_PROGRAM_ID);
    let consume_c = build_consume_license_ix(&consumer.pubkey(), &config_pda, &song_pda, &lic_c_pda, &consumer_pda, song_id, &user_c);
    let res = env.send_tx(consume_c, &[&consumer]);
    assert!(res.is_err(), "Must reject consumption when license count is 0");
}

#[test]
fn test_factory_consume_license_unauthorized_and_revoked_caller_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 2_000_000_000);
    let song_id = 1u64;
    let (song_pda, _) = find_song_pda(song_id, &FACTORY_PROGRAM_ID);
    env.send_tx(build_mint_song_ix(&creator.pubkey(), &config_pda, &song_pda, 10, 100, 10, 1, "ipfs://unauth-test"), &[&creator]).unwrap();

    let unauthorized_caller = Keypair::new();
    env.airdrop(&unauthorized_caller.pubkey(), 1_000_000_000);
    let user = Keypair::new().pubkey();
    let (lic_pda, _) = find_license_record_pda(song_id, &user, &FACTORY_PROGRAM_ID);

    // Call without authorized consumer PDA
    let dummy_acc = Keypair::new().pubkey();
    let consume_ix = build_consume_license_ix(&unauthorized_caller.pubkey(), &config_pda, &song_pda, &lic_pda, &dummy_acc, song_id, &user);
    let res = env.send_tx(consume_ix, &[&unauthorized_caller]);
    assert!(res.is_err(), "Unauthorized caller must be rejected");

    // Authorize then revoke consumer, then attempt consume
    let temp_consumer = Keypair::new();
    env.airdrop(&temp_consumer.pubkey(), 1_000_000_000);
    let (temp_consumer_pda, _) = find_authorized_consumer_pda(&temp_consumer.pubkey(), &FACTORY_PROGRAM_ID);
    env.send_tx(build_authorize_consumer_ix(&admin.pubkey(), &config_pda, &temp_consumer_pda, &temp_consumer.pubkey()), &[&admin]).unwrap();
    env.send_tx(build_revoke_consumer_ix(&admin.pubkey(), &config_pda, &temp_consumer_pda, &temp_consumer.pubkey()), &[&admin]).unwrap();

    let consume_revoked = build_consume_license_ix(&temp_consumer.pubkey(), &config_pda, &song_pda, &lic_pda, &temp_consumer_pda, song_id, &user);
    let res = env.send_tx(consume_revoked, &[&temp_consumer]);
    assert!(res.is_err(), "Revoked consumer must be rejected");
}

#[test]
fn test_factory_consume_license_on_non_remixable_song_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let consumer = Keypair::new();
    env.airdrop(&consumer.pubkey(), 2_000_000_000);
    let (consumer_pda, _) = find_authorized_consumer_pda(&consumer.pubkey(), &FACTORY_PROGRAM_ID);
    env.send_tx(build_authorize_consumer_ix(&admin.pubkey(), &config_pda, &consumer_pda, &consumer.pubkey()), &[&admin]).unwrap();

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 2_000_000_000);
    let song_id = 1u64;
    let (song_pda, _) = find_song_pda(song_id, &FACTORY_PROGRAM_ID);
    env.send_tx(build_mint_song_ix(&creator.pubkey(), &config_pda, &song_pda, 10, 100, 10, 0, "ipfs://non-remixable"), &[&creator]).unwrap();

    let user = Keypair::new().pubkey();
    let (lic_pda, _) = find_license_record_pda(song_id, &user, &FACTORY_PROGRAM_ID);
    let consume = build_consume_license_ix(&consumer.pubkey(), &config_pda, &song_pda, &lic_pda, &consumer_pda, song_id, &user);
    let res = env.send_tx(consume, &[&consumer]);
    assert!(res.is_err(), "Cannot consume license on non-remixable song");
}

#[test]
fn test_factory_mint_remix_direct_lifecycle_and_nested_prevention() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let consumer = Keypair::new();
    env.airdrop(&consumer.pubkey(), 3_000_000_000);
    let (consumer_pda, _) = find_authorized_consumer_pda(&consumer.pubkey(), &FACTORY_PROGRAM_ID);
    env.send_tx(build_authorize_consumer_ix(&admin.pubkey(), &config_pda, &consumer_pda, &consumer.pubkey()), &[&admin]).unwrap();

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 2_000_000_000);
    let parent_id = 1u64;
    let (parent_pda, _) = find_song_pda(parent_id, &FACTORY_PROGRAM_ID);
    env.send_tx(build_mint_song_ix(&creator.pubkey(), &config_pda, &parent_pda, 10, 100, 10, 1, "ipfs://parent-song"), &[&creator]).unwrap();

    let remixer = Keypair::new().pubkey();
    let remix_song_id = 2u64;
    let (remix_song_pda, bump) = find_song_pda(remix_song_id, &FACTORY_PROGRAM_ID);

    let mint_remix_ix = build_mint_remix_ix(
        &consumer.pubkey(),
        &config_pda,
        &parent_pda,
        &remix_song_pda,
        &consumer_pda,
        5,
        500,
        20,
        parent_id,
        &remixer,
        "ipfs://remix-song-v1",
    );
    let cu = env.send_tx(mint_remix_ix, &[&consumer]).expect("mint remix");
    assert!(cu < 25_000);

    let remix_acc = env.svm.get_account(&remix_song_pda).expect("remix song created");
    assert_eq!(remix_acc.data[0], SongAccount::DISCRIMINATOR);
    assert_eq!(remix_acc.data[1], bump);
    assert_eq!(remix_acc.data[2], 0); // remixable = 0 (remixes cannot be remixed)
    assert_eq!(remix_acc.data[3], 1); // is_remix = 1
    let parent_in_remix = u64::from_le_bytes(remix_acc.data[16..24].try_into().unwrap());
    assert_eq!(parent_in_remix, parent_id);
    assert_eq!(&remix_acc.data[48..80], remixer.as_ref());

    // Attempting to consume license on the remix must fail with RemixCannotBeRemixed
    let user = Keypair::new().pubkey();
    let (lic_pda, _) = find_license_record_pda(remix_song_id, &user, &FACTORY_PROGRAM_ID);
    let consume_on_remix = build_consume_license_ix(&consumer.pubkey(), &config_pda, &remix_song_pda, &lic_pda, &consumer_pda, remix_song_id, &user);
    let res = env.send_tx(consume_on_remix, &[&consumer]);
    assert!(res.is_err(), "Remix song cannot have licenses consumed for further remixing");

    // Attempting to mint a remix of a remix must fail with RemixCannotBeRemixed
    let (nested_remix_pda, _) = find_song_pda(3, &FACTORY_PROGRAM_ID);
    let nested_remix_ix = build_mint_remix_ix(
        &consumer.pubkey(),
        &config_pda,
        &remix_song_pda,
        &nested_remix_pda,
        &consumer_pda,
        1,
        100,
        10,
        remix_song_id,
        &remixer,
        "ipfs://nested-remix-illegal",
    );
    let res = env.send_tx(nested_remix_ix, &[&consumer]);
    assert!(res.is_err(), "Nested derivative remixes must be rejected");
}

#[test]
fn test_factory_update_pricing_and_metadata_uri() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 2_000_000_000);
    let song_id = 1u64;
    let (song_pda, _) = find_song_pda(song_id, &FACTORY_PROGRAM_ID);
    env.send_tx(build_mint_song_ix(&creator.pubkey(), &config_pda, &song_pda, 10, 100, 10, 1, "ipfs://orig-uri"), &[&creator]).unwrap();

    // Creator updates pricing
    let new_lic_price = 250_000u64;
    let new_str_price = 5_000u64;
    let update_price_ix = build_update_pricing_ix(&creator.pubkey(), &song_pda, song_id, new_lic_price, new_str_price);
    env.send_tx(update_price_ix, &[&creator]).expect("update pricing");

    let song_acc = env.svm.get_account(&song_pda).unwrap();
    let lic_p = u64::from_le_bytes(song_acc.data[32..40].try_into().unwrap());
    let str_p = u64::from_le_bytes(song_acc.data[40..48].try_into().unwrap());
    assert_eq!(lic_p, new_lic_price);
    assert_eq!(str_p, new_str_price);

    // Attacker cannot update pricing
    let attacker = Keypair::new();
    env.airdrop(&attacker.pubkey(), 1_000_000_000);
    let att_price_ix = build_update_pricing_ix(&attacker.pubkey(), &song_pda, song_id, 999, 999);
    let res = env.send_tx(att_price_ix, &[&attacker]);
    assert!(res.is_err(), "Attacker cannot update song pricing");

    // Creator updates metadata URI
    let new_uri = "ipfs://updated-metadata-v2-hq";
    let update_uri_ix = build_update_metadata_uri_ix(&creator.pubkey(), &song_pda, song_id, new_uri);
    env.send_tx(update_uri_ix, &[&creator]).expect("update metadata uri");

    let song_acc = env.svm.get_account(&song_pda).unwrap();
    let uri_len = u16::from_le_bytes(song_acc.data[4..6].try_into().unwrap()) as usize;
    assert_eq!(uri_len, new_uri.len());
    assert_eq!(&song_acc.data[80..80 + new_uri.len()], new_uri.as_bytes());

    // Attacker cannot update metadata URI
    let att_uri_ix = build_update_metadata_uri_ix(&attacker.pubkey(), &song_pda, song_id, "ipfs://hacked");
    let res = env.send_tx(att_uri_ix, &[&attacker]);
    assert!(res.is_err(), "Attacker cannot update metadata URI");
}

#[test]
fn test_factory_transfer_admin_lifecycle() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    env.send_tx(build_init_factory_ix(&admin.pubkey(), &config_pda), &[&admin]).unwrap();

    let new_admin = Keypair::new();
    env.airdrop(&new_admin.pubkey(), 2_000_000_000);

    let transfer_ix = build_transfer_admin_ix(&admin.pubkey(), &config_pda, &new_admin.pubkey());
    env.send_tx(transfer_ix, &[&admin]).expect("transfer admin");

    let cfg_acc = env.svm.get_account(&config_pda).unwrap();
    assert_eq!(&cfg_acc.data[8..40], new_admin.pubkey().as_ref());

    // Old admin can no longer authorize consumer
    let consumer = Keypair::new().pubkey();
    let (consumer_pda, _) = find_authorized_consumer_pda(&consumer, &FACTORY_PROGRAM_ID);
    let old_admin_ix = build_authorize_consumer_ix(&admin.pubkey(), &config_pda, &consumer_pda, &consumer);
    let res = env.send_tx(old_admin_ix, &[&admin]);
    assert!(res.is_err(), "Old admin must be revoked");

    // New admin can authorize consumer
    let new_admin_ix = build_authorize_consumer_ix(&new_admin.pubkey(), &config_pda, &consumer_pda, &consumer);
    env.send_tx(new_admin_ix, &[&new_admin]).expect("new admin authorize");
    assert!(env.svm.get_account(&consumer_pda).is_some());
}

#[test]
fn test_factory_mollusk_instruction_benchmark() {
    let deploy = deploy_path("rawchunks_factory");
    let factory_pubkey = to_pubkey(&FACTORY_PROGRAM_ID);
    std::env::set_var("SBF_OUT_DIR", deploy.parent().unwrap());
    let mollusk = Mollusk::new(&factory_pubkey, "rawchunks_factory");

    let admin = Pubkey::new_unique();
    let (config_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    let config_pubkey = to_pubkey(&config_pda);

    let ix = Instruction::new_with_bytes(
        factory_pubkey,
        &[0u8],
        vec![
            AccountMeta::new(admin, true),
            AccountMeta::new(config_pubkey, false),
            AccountMeta::new_readonly(to_pubkey(&SYSTEM_PROGRAM_ID), false),
        ],
    );

    let admin_account = Account::new(10_000_000_000, 0, &to_pubkey(&SYSTEM_PROGRAM_ID));
    let config_account = Account::new(0, 0, &to_pubkey(&SYSTEM_PROGRAM_ID));
    let system_account = keyed_account_for_system_program();

    let result = mollusk.process_instruction(
        &ix,
        &[
            (admin, admin_account),
            (config_pubkey, config_account),
            (system_account.0, system_account.1),
        ],
    );

    assert!(result.compute_units_consumed < 12_000, "Compute units: {}", result.compute_units_consumed);
}
