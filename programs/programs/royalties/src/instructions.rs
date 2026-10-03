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
