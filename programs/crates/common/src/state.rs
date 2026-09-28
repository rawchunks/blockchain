use pinocchio::{error::ProgramError, AccountView, Address};
use crate::{
    RawchunksError, MAX_METADATA_URI_LEN, SEED_CONSUMER, SEED_FACTORY, SEED_LICENSE, SEED_REMIX,
    SEED_REMIX_AUTHORITY, SEED_REMIX_COUNT, SEED_ROYALTIES, SEED_SONG,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoyaltiesConfig {
    pub discriminator: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
    pub admin: [u8; 32],
    pub treasury: [u8; 32],
    pub usdc_mint: [u8; 32],
}

impl RoyaltiesConfig {
    pub const DISCRIMINATOR: u8 = 1;
    pub const LEN: usize = 104;

    #[inline(always)]
    pub fn from_account_info<'a>(account: &'a mut AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { account.borrow_unchecked_mut() };
        if data[0] != Self::DISCRIMINATOR {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    #[inline(always)]
    pub fn from_account_info_init<'a>(account: &'a mut AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { account.borrow_unchecked_mut() };
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    #[inline(always)]
    pub fn from_account_info_readonly<'a>(account: &'a AccountView) -> Result<&'a Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { core::slice::from_raw_parts(account.data_ptr(), account.data_len()) };
        if data[0] != Self::DISCRIMINATOR {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }

    pub fn find_pda(program_id: &Address) -> (Address, u8) {
        Address::derive_program_address(&[SEED_ROYALTIES], program_id).unwrap()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactoryConfig {
    pub discriminator: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
    pub admin: [u8; 32],
    pub next_song_id: u64,
    pub total_songs: u64,
}

impl FactoryConfig {
    pub const DISCRIMINATOR: u8 = 1;
    pub const LEN: usize = 56;

    #[inline(always)]
    pub fn from_account_info<'a>(account: &'a mut AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { account.borrow_unchecked_mut() };
        if data[0] != Self::DISCRIMINATOR {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    #[inline(always)]
    pub fn from_account_info_init<'a>(account: &'a mut AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { account.borrow_unchecked_mut() };
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    #[inline(always)]
    pub fn from_account_info_readonly<'a>(account: &'a AccountView) -> Result<&'a Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { account.borrow_unchecked() };
        if data[0] != Self::DISCRIMINATOR {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }

    pub fn find_pda(program_id: &Address) -> (Address, u8) {
        Address::derive_program_address(&[SEED_FACTORY], program_id).unwrap()
    }
}
