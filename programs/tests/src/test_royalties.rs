use {
    crate::helpers::*,
    mollusk_svm::{program::keyed_account_for_system_program, Mollusk},
    rawchunks_common::{
        state::RoyaltiesConfig,
        BASIS_POINTS, TREASURY_SHARE_BPS,
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

fn build_init_royalties_ix(
    admin: &Address,
    config: &Address,
    treasury: &Address,
    usdc_mint: &Address,
) -> Instruction {
    let mut data = vec![0u8];
    data.extend_from_slice(treasury.as_ref());
    data.extend_from_slice(usdc_mint.as_ref());

    Instruction {
        program_id: ROYALTIES_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(*config, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

fn build_distribute_payment_ix(
    payer: &Address,
    payer_token: &Address,
    creator_token: &Address,
    treasury_token: &Address,
    config: &Address,
    token_program: &Address,
    song_id: u64,
    license_price: u64,
) -> Instruction {
    let mut data = vec![1u8];
    data.extend_from_slice(&song_id.to_le_bytes());
    data.extend_from_slice(&license_price.to_le_bytes());

    Instruction {
        program_id: ROYALTIES_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*payer_token, false),
            AccountMeta::new(*creator_token, false),
            AccountMeta::new(*treasury_token, false),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*token_program, false),
        ],
        data,
    }
}

fn build_set_usdc_ix(admin: &Address, config: &Address, new_usdc: &Address) -> Instruction {
    let mut data = vec![2u8];
    data.extend_from_slice(new_usdc.as_ref());

    Instruction {
        program_id: ROYALTIES_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*admin, true),
            AccountMeta::new(*config, false),
        ],
        data,
    }
}

fn build_set_treasury_ix(admin: &Address, config: &Address, new_treasury: &Address) -> Instruction {
    let mut data = vec![3u8];
    data.extend_from_slice(new_treasury.as_ref());

    Instruction {
        program_id: ROYALTIES_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*admin, true),
            AccountMeta::new(*config, false),
        ],
        data,
    }
}

fn build_transfer_admin_ix(admin: &Address, config: &Address, new_admin: &Address) -> Instruction {
    let mut data = vec![4u8];
    data.extend_from_slice(new_admin.as_ref());

    Instruction {
        program_id: ROYALTIES_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*admin, true),
            AccountMeta::new(*config, false),
        ],
        data,
    }
}

#[test]
fn test_royalties_initialize_and_state_verification() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, bump) = find_royalties_config_pda(&ROYALTIES_PROGRAM_ID);

    let treasury = Keypair::new().pubkey();
    let mint_authority = Keypair::new().pubkey();
    let usdc_mint = env.setup_mint(&mint_authority);

    let ix = build_init_royalties_ix(&admin.pubkey(), &config_pda, &treasury, &usdc_mint);
    let cu = env.send_tx(ix, &[&admin]).expect("init royalties");
    assert!(cu < 15_000, "Compute unit consumption exceeds budget: {}", cu);

    let account = env.svm.get_account(&config_pda).expect("account exists");
    assert_eq!(account.data.len(), RoyaltiesConfig::LEN);
    assert_eq!(account.data[0], RoyaltiesConfig::DISCRIMINATOR);
    assert_eq!(account.data[1], bump);
    assert_eq!(&account.data[8..40], admin.pubkey().as_ref());
    assert_eq!(&account.data[40..72], treasury.as_ref());
    assert_eq!(&account.data[72..104], usdc_mint.as_ref());
}

#[test]
fn test_royalties_duplicate_initialization_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_royalties_config_pda(&ROYALTIES_PROGRAM_ID);
    let treasury = Keypair::new().pubkey();
    let usdc_mint = env.setup_mint(&admin.pubkey());

    let ix1 = build_init_royalties_ix(&admin.pubkey(), &config_pda, &treasury, &usdc_mint);
    env.send_tx(ix1, &[&admin]).expect("first init");

    let ix2 = build_init_royalties_ix(&admin.pubkey(), &config_pda, &treasury, &usdc_mint);
    let res = env.send_tx(ix2, &[&admin]);
    assert!(res.is_err());
}

#[test]
fn test_royalties_init_invalid_pda_seeds_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let invalid_config = Keypair::new().pubkey();
    let treasury = Keypair::new().pubkey();
    let usdc_mint = env.setup_mint(&admin.pubkey());

    let ix = build_init_royalties_ix(&admin.pubkey(), &invalid_config, &treasury, &usdc_mint);
    let res = env.send_tx(ix, &[&admin]);
    assert!(res.is_err());
}

#[test]
fn test_royalties_init_missing_admin_signature_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let payer = env.payer();
    let (config_pda, _) = find_royalties_config_pda(&ROYALTIES_PROGRAM_ID);
    let treasury = Keypair::new().pubkey();
    let usdc_mint = env.setup_mint(&admin.pubkey());

    let mut ix = build_init_royalties_ix(&admin.pubkey(), &config_pda, &treasury, &usdc_mint);
    ix.accounts[0].is_signer = false;

    let res = env.send_tx(ix, &[&payer]);
    assert!(res.is_err());
}

#[test]
fn test_royalties_admin_controls_lifecycle() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_royalties_config_pda(&ROYALTIES_PROGRAM_ID);
    let treasury_initial = Keypair::new().pubkey();
    let usdc_initial = env.setup_mint(&admin.pubkey());

    let init_ix = build_init_royalties_ix(&admin.pubkey(), &config_pda, &treasury_initial, &usdc_initial);
    env.send_tx(init_ix, &[&admin]).unwrap();

    let new_usdc = env.setup_mint(&admin.pubkey());
    let set_usdc_ix = build_set_usdc_ix(&admin.pubkey(), &config_pda, &new_usdc);
    env.send_tx(set_usdc_ix, &[&admin]).unwrap();

    let account = env.svm.get_account(&config_pda).unwrap();
    assert_eq!(&account.data[72..104], new_usdc.as_ref());

    let new_treasury = Keypair::new().pubkey();
    let set_treasury_ix = build_set_treasury_ix(&admin.pubkey(), &config_pda, &new_treasury);
    env.send_tx(set_treasury_ix, &[&admin]).unwrap();

    let account = env.svm.get_account(&config_pda).unwrap();
    assert_eq!(&account.data[40..72], new_treasury.as_ref());

    let attacker = Keypair::new();
    env.airdrop(&attacker.pubkey(), 1_000_000_000);
    let malicious_usdc_ix = build_set_usdc_ix(&attacker.pubkey(), &config_pda, &new_usdc);
    let res = env.send_tx(malicious_usdc_ix, &[&attacker]);
    assert!(res.is_err());

    let malicious_treasury_ix = build_set_treasury_ix(&attacker.pubkey(), &config_pda, &new_treasury);
    let res = env.send_tx(malicious_treasury_ix, &[&attacker]);
    assert!(res.is_err());

    let new_admin = Keypair::new();
    env.airdrop(&new_admin.pubkey(), 1_000_000_000);
    let transfer_ix = build_transfer_admin_ix(&admin.pubkey(), &config_pda, &new_admin.pubkey());
    env.send_tx(transfer_ix, &[&admin]).unwrap();

    let account = env.svm.get_account(&config_pda).unwrap();
    assert_eq!(&account.data[8..40], new_admin.pubkey().as_ref());

    let old_admin_ix = build_set_usdc_ix(&admin.pubkey(), &config_pda, &usdc_initial);
    let res = env.send_tx(old_admin_ix, &[&admin]);
    assert!(res.is_err());

    let authorized_ix = build_set_usdc_ix(&new_admin.pubkey(), &config_pda, &usdc_initial);
    env.send_tx(authorized_ix, &[&new_admin]).unwrap();
    let account = env.svm.get_account(&config_pda).unwrap();
    assert_eq!(&account.data[72..104], usdc_initial.as_ref());
}

#[test]
fn test_royalties_payment_distribution_standard_split() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_royalties_config_pda(&ROYALTIES_PROGRAM_ID);

    let treasury_owner = Keypair::new().pubkey();
    let creator_owner = Keypair::new().pubkey();
    let payer_owner = Keypair::new();
    env.airdrop(&payer_owner.pubkey(), 2_000_000_000);

    let usdc_mint = env.setup_mint(&admin.pubkey());

    let payer_token = env.setup_token_account(&payer_owner.pubkey(), &usdc_mint, 50_000_000);
    let creator_token = env.setup_token_account(&creator_owner, &usdc_mint, 0);
    let treasury_token = env.setup_token_account(&treasury_owner, &usdc_mint, 0);

    let init_ix = build_init_royalties_ix(&admin.pubkey(), &config_pda, &treasury_owner, &usdc_mint);
    env.send_tx(init_ix, &[&admin]).unwrap();

    let price = 10_000_000u64;
    let song_id = 42u64;

    let dist_ix = build_distribute_payment_ix(
        &payer_owner.pubkey(),
        &payer_token,
        &creator_token,
        &treasury_token,
        &config_pda,
        &TOKEN_PROGRAM_ID,
        song_id,
        price,
    );
    let cu = env.send_tx(dist_ix, &[&payer_owner]).expect("distribute payment");
    assert!(cu < 35_000, "Compute unit limit exceeded: {}", cu);

    let expected_treasury = (price as u128 * TREASURY_SHARE_BPS as u128 / BASIS_POINTS as u128) as u64;
    let expected_creator = price - expected_treasury;

    assert_eq!(expected_treasury, 2_000_000);
    assert_eq!(expected_creator, 8_000_000);
    assert_eq!(expected_creator + expected_treasury, price);

    assert_eq!(env.get_token_balance(&payer_token), 40_000_000);
    assert_eq!(env.get_token_balance(&creator_token), 8_000_000);
    assert_eq!(env.get_token_balance(&treasury_token), 2_000_000);
}

#[test]
fn test_royalties_payment_distribution_zero_price_noop() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_royalties_config_pda(&ROYALTIES_PROGRAM_ID);

    let treasury_owner = Keypair::new().pubkey();
    let creator_owner = Keypair::new().pubkey();
    let payer_owner = Keypair::new();
    env.airdrop(&payer_owner.pubkey(), 2_000_000_000);

    let usdc_mint = env.setup_mint(&admin.pubkey());
    let payer_token = env.setup_token_account(&payer_owner.pubkey(), &usdc_mint, 100_000);
    let creator_token = env.setup_token_account(&creator_owner, &usdc_mint, 0);
    let treasury_token = env.setup_token_account(&treasury_owner, &usdc_mint, 0);

    let init_ix = build_init_royalties_ix(&admin.pubkey(), &config_pda, &treasury_owner, &usdc_mint);
    env.send_tx(init_ix, &[&admin]).unwrap();

    let dist_ix = build_distribute_payment_ix(
        &payer_owner.pubkey(),
        &payer_token,
        &creator_token,
        &treasury_token,
        &config_pda,
        &TOKEN_PROGRAM_ID,
        1,
        0,
    );
    let cu = env.send_tx(dist_ix, &[&payer_owner]).unwrap();
    assert!(cu < 10_000);

    assert_eq!(env.get_token_balance(&payer_token), 100_000);
    assert_eq!(env.get_token_balance(&creator_token), 0);
    assert_eq!(env.get_token_balance(&treasury_token), 0);
}

#[test]
fn test_royalties_odd_amount_splits_conservation_of_funds() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_royalties_config_pda(&ROYALTIES_PROGRAM_ID);

    let treasury_owner = Keypair::new().pubkey();
    let creator_owner = Keypair::new().pubkey();
    let payer_owner = Keypair::new();
    env.airdrop(&payer_owner.pubkey(), 2_000_000_000);

    let usdc_mint = env.setup_mint(&admin.pubkey());
    let payer_token = env.setup_token_account(&payer_owner.pubkey(), &usdc_mint, 100_000_000);
    let creator_token = env.setup_token_account(&creator_owner, &usdc_mint, 0);
    let treasury_token = env.setup_token_account(&treasury_owner, &usdc_mint, 0);

    let init_ix = build_init_royalties_ix(&admin.pubkey(), &config_pda, &treasury_owner, &usdc_mint);
    env.send_tx(init_ix, &[&admin]).unwrap();

    let odd_amounts = [1u64, 3u64, 7u64, 11u64, 99u64, 1337u64, 999_999u64];
    let mut total_payer_spent = 0u64;
    let mut total_creator_received = 0u64;
    let mut total_treasury_received = 0u64;

    for (i, &price) in odd_amounts.iter().enumerate() {
        let dist_ix = build_distribute_payment_ix(
            &payer_owner.pubkey(),
            &payer_token,
            &creator_token,
            &treasury_token,
            &config_pda,
            &TOKEN_PROGRAM_ID,
            (i + 1) as u64,
            price,
        );
        env.send_tx(dist_ix, &[&payer_owner]).unwrap();

        let treasury_fee = (price as u128 * TREASURY_SHARE_BPS as u128 / BASIS_POINTS as u128) as u64;
        let creator_share = price - treasury_fee;

        assert_eq!(creator_share + treasury_fee, price);
        total_payer_spent += price;
        total_creator_received += creator_share;
        total_treasury_received += treasury_fee;

        assert_eq!(env.get_token_balance(&creator_token), total_creator_received);
        assert_eq!(env.get_token_balance(&treasury_token), total_treasury_received);
        assert_eq!(env.get_token_balance(&payer_token), 100_000_000 - total_payer_spent);
    }

    assert_eq!(total_creator_received + total_treasury_received, total_payer_spent);
}

#[test]
fn test_royalties_usdc_not_set_rejected() {
    let mut env = TestEnv::new();
    let admin = env.admin();
    let (config_pda, _) = find_royalties_config_pda(&ROYALTIES_PROGRAM_ID);

    let treasury_owner = Keypair::new().pubkey();
    let zero_usdc = Address::default();

    let init_ix = build_init_royalties_ix(&admin.pubkey(), &config_pda, &treasury_owner, &zero_usdc);
    env.send_tx(init_ix, &[&admin]).unwrap();

    let payer_owner = Keypair::new();
    env.airdrop(&payer_owner.pubkey(), 1_000_000_000);
    let dummy_mint = env.setup_mint(&admin.pubkey());
    let payer_token = env.setup_token_account(&payer_owner.pubkey(), &dummy_mint, 10_000);
    let creator_token = env.setup_token_account(&Keypair::new().pubkey(), &dummy_mint, 0);
    let treasury_token = env.setup_token_account(&treasury_owner, &dummy_mint, 0);

    let dist_ix = build_distribute_payment_ix(
        &payer_owner.pubkey(),
        &payer_token,
        &creator_token,
        &treasury_token,
        &config_pda,
        &TOKEN_PROGRAM_ID,
        1,
        5000,
    );
    let res = env.send_tx(dist_ix, &[&payer_owner]);
    assert!(res.is_err());
}
