use {
    crate::helpers::*,
    mollusk_svm::{program::keyed_account_for_system_program, Mollusk},
    rawchunks_common::state::{
        LicenseRecord, RemixConfig, RemixRecord, SongAccount, SongRemixCount,
    },
    solana_account::Account,
    solana_address::Address,
    solana_instruction::{account_meta::AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_system_interface::program::ID as SYSTEM_PROGRAM_ID,
    spl_token_interface::ID as TOKEN_PROGRAM_ID,
};

fn build_init_remix_ix(
    admin: &Address,
    config: &Address,
    factory_prog: &Address,
    royalties_prog: &Address,
) -> Instruction {
    let mut data = vec![0u8];
    data.extend_from_slice(factory_prog.as_ref());
    data.extend_from_slice(royalties_prog.as_ref());

    Instruction {
        program_id: REMIX_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(*config, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

fn build_remix_song_ix(
    remixer: &Address,
    remix_authority: &Address,
    remix_config: &Address,
    factory_prog: &Address,
    factory_config: &Address,
    original_song: &Address,
    remix_song: &Address,
    license_record: &Address,
    remix_consumer: &Address,
    remix_count: &Address,
    remix_record: &Address,
    original_song_id: u64, license_amount: u64, license_price: u64,
    stream_price: u64, uri: &str,
) -> Instruction {
    let mut data = vec![1u8];
    data.extend_from_slice(&original_song_id.to_le_bytes());
    data.extend_from_slice(&license_amount.to_le_bytes());
    data.extend_from_slice(&license_price.to_le_bytes());
    data.extend_from_slice(&stream_price.to_le_bytes());
    let uri_bytes = uri.as_bytes();
    data.extend_from_slice(&(uri_bytes.len() as u16).to_le_bytes());
    data.extend_from_slice(uri_bytes);

    Instruction {
        program_id: REMIX_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*remixer, true),
            AccountMeta::new(*remix_authority, false),
            AccountMeta::new(*remix_config, false),
            AccountMeta::new_readonly(*factory_prog, false),
            AccountMeta::new(*factory_config, false),
            AccountMeta::new_readonly(*original_song, false),
            AccountMeta::new(*remix_song, false),
            AccountMeta::new_readonly(*license_record, false),
            AccountMeta::new_readonly(*remix_consumer, false),
            AccountMeta::new(*remix_count, false),
            AccountMeta::new(*remix_record, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

fn build_buy_license_ix(
    buyer: &Address,
    authority: &Address,
    config: &Address,
    factory: &Address,
    royalties: &Address,
    factory_config: &Address,
    song: &Address,
    license: &Address,
    consumer: &Address,
    payer_token: &Address,
    creator_token: &Address,
    treasury_token: &Address,
    royalties_config: &Address,
    token: &Address,
    song_id: u64,
) -> Instruction {
    let mut data = vec![4u8];
    data.extend_from_slice(&song_id.to_le_bytes());
    Instruction {
        program_id: REMIX_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*buyer, true),
            AccountMeta::new(*authority, false),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*factory, false),
            AccountMeta::new_readonly(*royalties, false),
            AccountMeta::new_readonly(*factory_config, false),
            AccountMeta::new(*song, false),
            AccountMeta::new(*license, false),
            AccountMeta::new_readonly(*consumer, false),
            AccountMeta::new(*payer_token, false),
            AccountMeta::new(*creator_token, false),
            AccountMeta::new(*treasury_token, false),
            AccountMeta::new_readonly(*royalties_config, false),
            AccountMeta::new_readonly(*token, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

fn build_update_remix_programs_ix(
    admin: &Address,
    config: &Address,
    new_factory: &Address,
    new_royalties: &Address,
) -> Instruction {
    let mut data = vec![2u8];
    data.extend_from_slice(new_factory.as_ref());
    data.extend_from_slice(new_royalties.as_ref());

    Instruction {
        program_id: REMIX_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*admin, true),
            AccountMeta::new(*config, false),
        ],
        data,
    }
}

fn build_transfer_remix_admin_ix(
    admin: &Address,
    config: &Address,
    new_admin: &Address,
) -> Instruction {
    let mut data = vec![3u8];
    data.extend_from_slice(new_admin.as_ref());

    Instruction {
        program_id: REMIX_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*admin, true),
            AccountMeta::new(*config, false),
        ],
        data,
    }
}

#[test]
fn test_remix_initialize_and_state_verification() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, bump) = find_remix_config_pda(&REMIX_PROGRAM_ID);
    let (_, auth_bump) = find_remix_authority_pda(&REMIX_PROGRAM_ID);

    let ix = build_init_remix_ix(
        &admin.pubkey(),
        &config_pda,
        &FACTORY_PROGRAM_ID,
        &ROYALTIES_PROGRAM_ID,
    );
    let cu = env.send_tx(ix, &[&admin]).expect("init remix");
    assert!(cu < 15_000, "Compute units: {}", cu);

    let account = env.svm.get_account(&config_pda).expect("account exists");
    assert_eq!(account.data.len(), RemixConfig::LEN);
    assert_eq!(account.data[0], RemixConfig::DISCRIMINATOR);
    assert_eq!(account.data[1], bump);
    assert_eq!(account.data[2], auth_bump);
    let total_remixes = u64::from_le_bytes(account.data[8..16].try_into().unwrap());
    assert_eq!(total_remixes, 0);
    assert_eq!(&account.data[16..48], admin.pubkey().as_ref());
    assert_eq!(&account.data[48..80], FACTORY_PROGRAM_ID.as_ref());
    assert_eq!(&account.data[80..112], ROYALTIES_PROGRAM_ID.as_ref());
}

#[test]
fn test_remix_duplicate_init_and_invalid_seeds_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_remix_config_pda(&REMIX_PROGRAM_ID);

    let ix1 = build_init_remix_ix(
        &admin.pubkey(),
        &config_pda,
        &FACTORY_PROGRAM_ID,
        &ROYALTIES_PROGRAM_ID,
    );
    env.send_tx(ix1, &[&admin]).unwrap();

    let ix2 = build_init_remix_ix(
        &admin.pubkey(),
        &config_pda,
        &FACTORY_PROGRAM_ID,
        &ROYALTIES_PROGRAM_ID,
    );
    let res = env.send_tx(ix2, &[&admin]);
    assert!(res.is_err());

    let invalid_config = Keypair::new().pubkey();
    let ix3 = build_init_remix_ix(
        &admin.pubkey(),
        &invalid_config,
        &FACTORY_PROGRAM_ID,
        &ROYALTIES_PROGRAM_ID,
    );
    let res = env.send_tx(ix3, &[&admin]);
    assert!(res.is_err());
}

#[test]
fn test_remix_song_full_three_program_orchestration() {
    let mut env = TestEnv::new();
    let admin = env.admin();

    let (royalties_cfg_pda, _) = find_royalties_config_pda(&ROYALTIES_PROGRAM_ID);
    let treasury_owner = Keypair::new().pubkey();
    let usdc_mint = env.setup_mint(&admin.pubkey());

    let mut init_roy_data = vec![0u8];
    init_roy_data.extend_from_slice(treasury_owner.as_ref());
    init_roy_data.extend_from_slice(usdc_mint.as_ref());
    let init_roy_ix = Instruction {
        program_id: ROYALTIES_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(royalties_cfg_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: init_roy_data,
    };
    env.send_tx(init_roy_ix, &[&admin]).unwrap();

    let (factory_cfg_pda, _) = find_factory_config_pda(&FACTORY_PROGRAM_ID);
    let init_fac_ix = Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(factory_cfg_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: vec![0u8],
    };
    env.send_tx(init_fac_ix, &[&admin]).unwrap();

    let (remix_cfg_pda, _) = find_remix_config_pda(&REMIX_PROGRAM_ID);
    let (remix_auth_pda, _) = find_remix_authority_pda(&REMIX_PROGRAM_ID);
    let init_rem_ix = build_init_remix_ix(
        &admin.pubkey(),
        &remix_cfg_pda,
        &FACTORY_PROGRAM_ID,
        &ROYALTIES_PROGRAM_ID,
    );
    env.send_tx(init_rem_ix, &[&admin]).unwrap();

    let (remix_consumer_pda, _) = find_authorized_consumer_pda(&remix_auth_pda, &FACTORY_PROGRAM_ID);
    let mut auth_data = vec![6u8];
    auth_data.extend_from_slice(remix_auth_pda.as_ref());
    let auth_ix = Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new_readonly(factory_cfg_pda, false),
            AccountMeta::new(remix_consumer_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: auth_data,
    };
    env.send_tx(auth_ix, &[&admin]).unwrap();

    env.airdrop(&remix_auth_pda, 500_000_000);

    let creator = Keypair::new();
    env.airdrop(&creator.pubkey(), 2_000_000_000);
    let song_id = 1u64;
    let (orig_song_pda, _) = find_song_pda(song_id, &FACTORY_PROGRAM_ID);

    let orig_price = 10_000_000u64; // 10 USDC
    let orig_licenses = 2u64;
    let orig_uri = "ipfs://original-anthem-track";

    let mut mint_data = vec![1u8];
    mint_data.extend_from_slice(&orig_licenses.to_le_bytes());
    mint_data.extend_from_slice(&orig_price.to_le_bytes());
    mint_data.extend_from_slice(&50_000u64.to_le_bytes());
    mint_data.push(1); // remixable = 1
    mint_data.extend_from_slice(&(orig_uri.len() as u16).to_le_bytes());
    mint_data.extend_from_slice(orig_uri.as_bytes());

    let mint_song_ix = Instruction {
        program_id: FACTORY_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(creator.pubkey(), true),
            AccountMeta::new(factory_cfg_pda, false),
            AccountMeta::new(orig_song_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: mint_data,
    };
    env.send_tx(mint_song_ix, &[&creator]).unwrap();

    let remixer = Keypair::new();
    env.airdrop(&remixer.pubkey(), 2_000_000_000);

    let remixer_token = env.setup_token_account(&remixer.pubkey(), &usdc_mint, 50_000_000);
    let creator_token = env.setup_token_account(&creator.pubkey(), &usdc_mint, 0);
    let treasury_token = env.setup_token_account(&treasury_owner, &usdc_mint, 0);

    let remixed_song_id = 2u64;
    let (remix_song_pda, _) = find_song_pda(remixed_song_id, &FACTORY_PROGRAM_ID);
    let (lic_rec_pda, _) = find_license_record_pda(song_id, &remixer.pubkey(), &FACTORY_PROGRAM_ID);
    let (count_pda, count_bump) = find_song_remix_count_pda(song_id, &REMIX_PROGRAM_ID);
    let (rec_pda, rec_bump) = find_remix_record_pda(song_id, 0, &REMIX_PROGRAM_ID);

    let remix_uri = "ipfs://remix-track-cyberpunk-edition";
    let purchase = build_buy_license_ix(
        &remixer.pubkey(),
        &remix_auth_pda,
        &remix_cfg_pda,
        &FACTORY_PROGRAM_ID,
        &ROYALTIES_PROGRAM_ID,
        &factory_cfg_pda,
        &orig_song_pda,
        &lic_rec_pda,
        &remix_consumer_pda,
        &remixer_token,
        &creator_token,
        &treasury_token,
        &royalties_cfg_pda,
        &TOKEN_PROGRAM_ID,
        song_id,
    );
    env.send_tx(purchase, &[&remixer]).expect("buy license");
    assert!(env.svm.get_account(&remix_song_pda).is_none());
    let remix_ix = build_remix_song_ix(
        &remixer.pubkey(),
        &remix_auth_pda,
        &remix_cfg_pda,
        &FACTORY_PROGRAM_ID,
        &factory_cfg_pda,
        &orig_song_pda,
        &remix_song_pda,
        &lic_rec_pda,
        &remix_consumer_pda,
        &count_pda,
        &rec_pda,
        song_id,
        5,
        1_000_000,
        25_000,
        remix_uri,
    );

    let cu = env.send_tx(remix_ix, &[&remixer]).expect("remix song 1");
    assert!(cu < 75_000, "Compute budget exceeded: {}", cu);

    assert_eq!(env.get_token_balance(&remixer_token), 40_000_000);
    assert_eq!(env.get_token_balance(&creator_token), 8_000_000);
    assert_eq!(env.get_token_balance(&treasury_token), 2_000_000);

    let orig_acc = env.svm.get_account(&orig_song_pda).unwrap();
    let rem_licenses = u64::from_le_bytes(orig_acc.data[24..32].try_into().unwrap());
    assert_eq!(rem_licenses, 1);

    let lic_acc = env.svm.get_account(&lic_rec_pda).expect("license record created");
    assert_eq!(lic_acc.data[0], LicenseRecord::DISCRIMINATOR);
    assert_eq!(&lic_acc.data[16..48], remixer.pubkey().as_ref());

    let remix_acc = env.svm.get_account(&remix_song_pda).expect("remix song created");
    assert_eq!(remix_acc.data[0], SongAccount::DISCRIMINATOR);
    assert_eq!(remix_acc.data[2], 0); // remixable = 0
    assert_eq!(remix_acc.data[3], 1); // is_remix = 1
    let parent_id = u64::from_le_bytes(remix_acc.data[16..24].try_into().unwrap());
    assert_eq!(parent_id, song_id);
    assert_eq!(&remix_acc.data[48..80], remixer.pubkey().as_ref());

    let count_acc = env.svm.get_account(&count_pda).expect("count account created");
    assert_eq!(count_acc.data[0], SongRemixCount::DISCRIMINATOR);
    assert_eq!(count_acc.data[1], count_bump);
    let count_val = u64::from_le_bytes(count_acc.data[16..24].try_into().unwrap());
    assert_eq!(count_val, 1);

    let rec_acc = env.svm.get_account(&rec_pda).expect("remix record created");
    assert_eq!(rec_acc.data[0], RemixRecord::DISCRIMINATOR);
    assert_eq!(rec_acc.data[1], rec_bump);
    let rec_orig_id = u64::from_le_bytes(rec_acc.data[8..16].try_into().unwrap());
    assert_eq!(rec_orig_id, song_id);
    let rec_idx = u64::from_le_bytes(rec_acc.data[16..24].try_into().unwrap());
    assert_eq!(rec_idx, 0);
    let rec_remixed_id = u64::from_le_bytes(rec_acc.data[24..32].try_into().unwrap());
    assert_eq!(rec_remixed_id, remixed_song_id);
    assert_eq!(&rec_acc.data[32..64], remixer.pubkey().as_ref());

    let rem_cfg_acc = env.svm.get_account(&remix_cfg_pda).unwrap();
    let tot_remixes = u64::from_le_bytes(rem_cfg_acc.data[8..16].try_into().unwrap());
    assert_eq!(tot_remixes, 1);

    let remixer_2 = Keypair::new();
    env.airdrop(&remixer_2.pubkey(), 2_000_000_000);
    let remixer_2_token = env.setup_token_account(&remixer_2.pubkey(), &usdc_mint, 20_000_000);

    let remixed_song_2_id = 3u64;
    let (remix_song_2_pda, _) = find_song_pda(remixed_song_2_id, &FACTORY_PROGRAM_ID);
    let (lic_rec_2_pda, _) = find_license_record_pda(song_id, &remixer_2.pubkey(), &FACTORY_PROGRAM_ID);
    let (rec_2_pda, _) = find_remix_record_pda(song_id, 1, &REMIX_PROGRAM_ID);

    let purchase = build_buy_license_ix(
        &remixer_2.pubkey(),
        &remix_auth_pda,
        &remix_cfg_pda,
        &FACTORY_PROGRAM_ID,
        &ROYALTIES_PROGRAM_ID,
        &factory_cfg_pda,
        &orig_song_pda,
        &lic_rec_2_pda,
        &remix_consumer_pda,
        &remixer_2_token,
        &creator_token,
        &treasury_token,
        &royalties_cfg_pda,
        &TOKEN_PROGRAM_ID,
        song_id,
    );
    env.send_tx(purchase, &[&remixer_2]).expect("buy license");
    let remix_2_ix = build_remix_song_ix(
        &remixer_2.pubkey(),
        &remix_auth_pda,
        &remix_cfg_pda,
        &FACTORY_PROGRAM_ID,
        &factory_cfg_pda,
        &orig_song_pda,
        &remix_song_2_pda,
        &lic_rec_2_pda,
        &remix_consumer_pda,
        &count_pda,
        &rec_2_pda,
        song_id,
        3,
        500_000,
        15_000,
        "ipfs://remix-2-lofi-beats",
    );
    env.send_tx(remix_2_ix, &[&remixer_2]).expect("remix song 2");

    assert_eq!(env.get_token_balance(&remixer_2_token), 10_000_000);
    assert_eq!(env.get_token_balance(&creator_token), 16_000_000);
    assert_eq!(env.get_token_balance(&treasury_token), 4_000_000);

    let count_acc_2 = env.svm.get_account(&count_pda).unwrap();
    let count_val_2 = u64::from_le_bytes(count_acc_2.data[16..24].try_into().unwrap());
    assert_eq!(count_val_2, 2);

    let orig_acc_after = env.svm.get_account(&orig_song_pda).unwrap();
    let rem_licenses_after = u64::from_le_bytes(orig_acc_after.data[24..32].try_into().unwrap());
    assert_eq!(rem_licenses_after, 0);

    env.svm.expire_blockhash();
    let repeat_purchase = build_buy_license_ix(
        &remixer.pubkey(),
        &remix_auth_pda,
        &remix_cfg_pda,
        &FACTORY_PROGRAM_ID,
        &ROYALTIES_PROGRAM_ID,
        &factory_cfg_pda,
        &orig_song_pda,
        &lic_rec_pda,
        &remix_consumer_pda,
        &remixer_token,
        &creator_token,
        &treasury_token,
        &royalties_cfg_pda,
        &TOKEN_PROGRAM_ID,
        song_id,
    );
    env.send_tx(repeat_purchase, &[&remixer]).expect("reuse license");
    assert_eq!(env.get_token_balance(&remixer_token), 40_000_000);
    let (variation_song, _) = find_song_pda(4, &FACTORY_PROGRAM_ID);
    let (variation_record, _) = find_remix_record_pda(song_id, 2, &REMIX_PROGRAM_ID);
    let variation = build_remix_song_ix(&remixer.pubkey(), &remix_auth_pda, &remix_cfg_pda,
        &FACTORY_PROGRAM_ID, &factory_cfg_pda, &orig_song_pda, &variation_song, &lic_rec_pda,
        &remix_consumer_pda, &count_pda, &variation_record, song_id, 1, 0, 0, "ipfs://variation");
    env.send_tx(variation, &[&remixer]).expect("mint another variation");
    assert_eq!(env.get_token_balance(&remixer_token), 40_000_000);
    assert_eq!(env.get_token_balance(&creator_token), 16_000_000);
    assert_eq!(env.get_token_balance(&treasury_token), 4_000_000);
    let count = env.svm.get_account(&count_pda).unwrap();
    assert_eq!(u64::from_le_bytes(count.data[16..24].try_into().unwrap()), 3);

    let remixer_3 = Keypair::new();
    env.airdrop(&remixer_3.pubkey(), 2_000_000_000);
    let remixer_3_token = env.setup_token_account(&remixer_3.pubkey(), &usdc_mint, 20_000_000);

    let (remix_song_3_pda, _) = find_song_pda(5, &FACTORY_PROGRAM_ID);
    let (lic_rec_3_pda, _) = find_license_record_pda(song_id, &remixer_3.pubkey(), &FACTORY_PROGRAM_ID);
    let (rec_3_pda, _) = find_remix_record_pda(song_id, 2, &REMIX_PROGRAM_ID);

    let remix_3_ix = build_remix_song_ix(
        &remixer_3.pubkey(),
        &remix_auth_pda,
        &remix_cfg_pda,
        &FACTORY_PROGRAM_ID,
        &factory_cfg_pda,
        &orig_song_pda,
        &remix_song_3_pda,
        &lic_rec_3_pda,
        &remix_consumer_pda,
        &count_pda,
        &rec_3_pda,
        song_id,
        1,
        100_000,
        5_000,
        "ipfs://remix-3-failed",
    );
    let res = env.send_tx(remix_3_ix, &[&remixer_3]);
    assert!(res.is_err(), "Must reject an unlicensed remix");
    assert_eq!(env.get_token_balance(&remixer_3_token), 20_000_000, "Remixer 3 was not charged");

    let (nested_pda, _) = find_song_pda(4, &FACTORY_PROGRAM_ID);
    let (nested_lic_pda, _) = find_license_record_pda(remixed_song_id, &remixer_3.pubkey(), &FACTORY_PROGRAM_ID);
    let (nested_count_pda, _) = find_song_remix_count_pda(remixed_song_id, &REMIX_PROGRAM_ID);
    let (nested_rec_pda, _) = find_remix_record_pda(remixed_song_id, 0, &REMIX_PROGRAM_ID);

    let nested_remix_ix = build_remix_song_ix(
        &remixer_3.pubkey(),
        &remix_auth_pda,
        &remix_cfg_pda,
        &FACTORY_PROGRAM_ID,
        &factory_cfg_pda,
        &remix_song_pda,
        &nested_pda,
        &nested_lic_pda,
        &remix_consumer_pda,
        &nested_count_pda,
        &nested_rec_pda,
        remixed_song_id,
        1,
        100_000,
        5_000,
        "ipfs://nested-derivative-fail",
    );
    let res = env.send_tx(nested_remix_ix, &[&remixer_3]);
    assert!(res.is_err(), "Must reject nested remix of a remix");
}
