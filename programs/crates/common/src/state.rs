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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SongAccount {
    pub discriminator: u8,
    pub bump: u8,
    pub remixable: u8,
    pub is_remix: u8,
    pub uri_len: u16,
    pub _padding: [u8; 2],
    pub song_id: u64,
    pub parent_song_id: u64,
    pub license_amount: u64,
    pub license_price: u64,
    pub stream_price: u64,
    pub creator: [u8; 32],
    pub metadata_uri: [u8; MAX_METADATA_URI_LEN],
}

impl SongAccount {
    pub const DISCRIMINATOR: u8 = 2;
    pub const LEN: usize = 280;

    #[inline(always)]
    pub fn from_account_info<'a>(account: &'a mut AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { account.borrow_unchecked_mut() };
        if data[0] != Self::DISCRIMINATOR {
            return Err(RawchunksError::SongNotFound.into());
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
            return Err(RawchunksError::SongNotFound.into());
        }
        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }

    pub fn find_pda(song_id: u64, program_id: &Address) -> (Address, u8) {
        let song_id_bytes = song_id.to_le_bytes();
        Address::derive_program_address(&[SEED_SONG, &song_id_bytes], program_id).unwrap()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthorizedConsumer {
    pub discriminator: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
    pub consumer: [u8; 32],
}

impl AuthorizedConsumer {
    pub const DISCRIMINATOR: u8 = 3;
    pub const LEN: usize = 40;

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
            return Err(RawchunksError::NotAuthorized.into());
        }
        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }

    pub fn find_pda(consumer: &[u8; 32], program_id: &Address) -> (Address, u8) {
        Address::derive_program_address(&[SEED_CONSUMER, consumer.as_ref()], program_id).unwrap()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LicenseRecord {
    pub discriminator: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
    pub song_id: u64,
    pub user: [u8; 32],
}

impl LicenseRecord {
    pub const DISCRIMINATOR: u8 = 4;
    pub const LEN: usize = 48;

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

    pub fn find_pda(song_id: u64, user: &[u8; 32], program_id: &Address) -> (Address, u8) {
        let song_id_bytes = song_id.to_le_bytes();
        Address::derive_program_address(&[SEED_LICENSE, &song_id_bytes, user.as_ref()], program_id).unwrap()
    }
}
