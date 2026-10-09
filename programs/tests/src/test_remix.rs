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
