use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use pinocchio_log::log;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{
    instructions::Transfer,
    state::Account as PinocchioTokenAccount,
    TokenInterface, TokenProgram,
};
use rawchunks_common::{
    RawchunksError, BASIS_POINTS, SEED_ROYALTIES, TREASURY_FEE_BPS,
};

use crate::state::RoyaltiesConfig;

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

    if data.len() < 64 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let treasury: [u8; 32] = data[0..32].try_into().map_err(|_| RawchunksError::InvalidInstructionData)?;
    let usdc_mint: [u8; 32] = data[32..64].try_into().map_err(|_| RawchunksError::InvalidInstructionData)?;

    let (expected_pda, bump) = RoyaltiesConfig::find_pda(program_id);
    if config_account.address() != &expected_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    if config_account.data_len() > 0 {
        return Err(RawchunksError::AlreadyInitialized.into());
    }

    let bump_bytes = [bump];
    let seeds = [
        Seed::from(SEED_ROYALTIES),
        Seed::from(&bump_bytes),
    ];
    let signer = Signer::from(&seeds);

    CreateAccount::with_minimum_balance(
        admin,
        config_account,
        RoyaltiesConfig::LEN as u64,
        program_id,
        None,
    )?
    .invoke_signed(&[signer])?;

    let mut admin_address = [0u8; 32];
    admin_address.copy_from_slice(admin.address().as_ref());

    let config = RoyaltiesConfig::from_account_info_init(config_account)?;
    config.discriminator = RoyaltiesConfig::DISCRIMINATOR;
    config.bump = bump;
    config._padding = [0u8; 6];
    config.admin = admin_address;
    config.treasury = treasury;
    config.usdc_mint = usdc_mint;

    log!("Royalties initialized successfully");

    Ok(())
}

pub fn process_distribute_license_payment(
    program_id: &Address,
    accounts: &[AccountView],
    data: &[u8],
) -> ProgramResult {
    let [payer, payer_token, creator_token, treasury_token, config_account, token_program, _rem @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    if data.len() < 16 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let song_id = u64::from_le_bytes(data[0..8].try_into().map_err(|_| RawchunksError::InvalidInstructionData)?);
    let license_price = u64::from_le_bytes(data[8..16].try_into().map_err(|_| RawchunksError::InvalidInstructionData)?);

    if license_price == 0 {
        log!("License price is 0; skipping token transfer");
        return Ok(());
    }

    let (expected_pda, _) = RoyaltiesConfig::find_pda(program_id);
    if config_account.address() != &expected_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    TokenProgram::verify(token_program.address())?;

    let config = RoyaltiesConfig::from_account_info_readonly(config_account)?;
    if config.usdc_mint == [0u8; 32] {
        return Err(RawchunksError::UsdcNotSet.into());
    }

    let (payer_mint, payer_owner) = unsafe {
        let acc = PinocchioTokenAccount::from_account_view_unchecked(payer_token)?;
        (*acc.mint(), *acc.owner())
    };
    let (creator_mint, creator_owner) = unsafe {
        let acc = PinocchioTokenAccount::from_account_view_unchecked(creator_token)?;
        (*acc.mint(), *acc.owner())
    };
    let (treasury_mint, treasury_owner) = unsafe {
        let acc = PinocchioTokenAccount::from_account_view_unchecked(treasury_token)?;
        (*acc.mint(), *acc.owner())
    };

    if payer_owner.as_ref() != payer.address().as_ref() {
        return Err(RawchunksError::NotAuthorized.into());
    }

    if payer_mint.as_ref() != &config.usdc_mint
        || creator_mint.as_ref() != &config.usdc_mint
        || treasury_mint.as_ref() != &config.usdc_mint
    {
        return Err(RawchunksError::InvalidAddress.into());
    }

    if treasury_owner.as_ref() != &config.treasury {
        return Err(RawchunksError::InvalidAddress.into());
    }

    if data.len() >= 48 {
        let expected_creator = &data[16..48];
        if creator_owner.as_ref() != expected_creator {
            return Err(RawchunksError::InvalidAddress.into());
        }
    }

    let treasury_fee = license_price
        .checked_mul(TREASURY_FEE_BPS)
        .ok_or(RawchunksError::MathOverflow)?
        / BASIS_POINTS;
    let creator_share = license_price
        .checked_sub(treasury_fee)
        .ok_or(RawchunksError::MathOverflow)?;

    if creator_share > 0 {
        Transfer::new(payer_token, creator_token, payer, creator_share)
            .invoke_with_program(token_program.address())?;
    }

    if treasury_fee > 0 {
        Transfer::new(payer_token, treasury_token, payer, treasury_fee)
            .invoke_with_program(token_program.address())?;
    }

    log!(
        "LicensePaymentDistributed: song_id={}, price={}, creator_share={}, treasury_fee={}",
        song_id,
        license_price,
        creator_share,
        treasury_fee
    );

    Ok(())
}

pub fn process_set_usdc(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let [admin, config_account, _rem @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !admin.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    let (expected_pda, _) = RoyaltiesConfig::find_pda(program_id);
    if config_account.address() != &expected_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    if data.len() < 32 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let config = RoyaltiesConfig::from_account_info(config_account)?;
    if config.admin.as_ref() != admin.address().as_ref() {
        return Err(RawchunksError::NotAdmin.into());
    }

    let new_usdc: [u8; 32] = data[0..32].try_into().map_err(|_| RawchunksError::InvalidInstructionData)?;
    config.usdc_mint = new_usdc;

    log!("USDC mint updated");

    Ok(())
}

pub fn process_set_treasury(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let [admin, config_account, _rem @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !admin.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    let (expected_pda, _) = RoyaltiesConfig::find_pda(program_id);
    if config_account.address() != &expected_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    if data.len() < 32 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let config = RoyaltiesConfig::from_account_info(config_account)?;
    if config.admin.as_ref() != admin.address().as_ref() {
        return Err(RawchunksError::NotAdmin.into());
    }

    let new_treasury: [u8; 32] = data[0..32].try_into().map_err(|_| RawchunksError::InvalidInstructionData)?;
    config.treasury = new_treasury;

    log!("Treasury updated");

    Ok(())
}

pub fn process_transfer_admin(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let [admin, config_account, _rem @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !admin.is_signer() {
        return Err(RawchunksError::MissingRequiredSignature.into());
    }

    let (expected_pda, _) = RoyaltiesConfig::find_pda(program_id);
    if config_account.address() != &expected_pda {
        return Err(RawchunksError::InvalidSeeds.into());
    }

    if data.len() < 32 {
        return Err(RawchunksError::InvalidInstructionData.into());
    }

    let config = RoyaltiesConfig::from_account_info(config_account)?;
    if config.admin.as_ref() != admin.address().as_ref() {
        return Err(RawchunksError::NotAdmin.into());
    }

    let new_admin: [u8; 32] = data[0..32].try_into().map_err(|_| RawchunksError::InvalidInstructionData)?;
    config.admin = new_admin;

    log!("Royalties admin transferred");

    Ok(())
}
