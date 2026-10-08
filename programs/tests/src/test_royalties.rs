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
