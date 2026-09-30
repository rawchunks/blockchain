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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemixConfig {
    pub discriminator: u8,
    pub bump: u8,
    pub authority_bump: u8,
    pub _padding: [u8; 5],
    pub total_remixes: u64,
    pub admin: [u8; 32],
    pub factory_program: [u8; 32],
    pub royalties_program: [u8; 32],
}

impl RemixConfig {
    pub const DISCRIMINATOR: u8 = 1;
    pub const LEN: usize = 112;

    #[inline(always)]
    pub fn from_account_info<'a>(account: &'a AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { core::slice::from_raw_parts_mut(account.data_ptr() as *mut u8, account.data_len()) };
        if data[0] != Self::DISCRIMINATOR {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    #[inline(always)]
    pub fn from_account_info_init<'a>(account: &'a AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { core::slice::from_raw_parts_mut(account.data_ptr() as *mut u8, account.data_len()) };
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
        Address::derive_program_address(&[SEED_REMIX], program_id).unwrap()
    }

    pub fn find_authority_pda(program_id: &Address) -> (Address, u8) {
        Address::derive_program_address(&[SEED_REMIX_AUTHORITY], program_id).unwrap()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SongRemixCount {
    pub discriminator: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
    pub original_song_id: u64,
    pub count: u64,
}

impl SongRemixCount {
    pub const DISCRIMINATOR: u8 = 2;
    pub const LEN: usize = 24;

    #[inline(always)]
    pub fn from_account_info<'a>(account: &'a AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { core::slice::from_raw_parts_mut(account.data_ptr() as *mut u8, account.data_len()) };
        if data[0] != Self::DISCRIMINATOR {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    #[inline(always)]
    pub fn from_account_info_init<'a>(account: &'a AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { core::slice::from_raw_parts_mut(account.data_ptr() as *mut u8, account.data_len()) };
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

    pub fn find_pda(original_song_id: u64, program_id: &Address) -> (Address, u8) {
        let song_id_bytes = original_song_id.to_le_bytes();
        Address::derive_program_address(&[SEED_REMIX_COUNT, &song_id_bytes], program_id).unwrap()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemixRecord {
    pub discriminator: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
    pub original_song_id: u64,
    pub remix_index: u64,
    pub remix_song_id: u64,
    pub remixer: [u8; 32],
}

impl RemixRecord {
    pub const DISCRIMINATOR: u8 = 3;
    pub const LEN: usize = 64;

    #[inline(always)]
    pub fn from_account_info<'a>(account: &'a AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { core::slice::from_raw_parts_mut(account.data_ptr() as *mut u8, account.data_len()) };
        if data[0] != Self::DISCRIMINATOR {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    #[inline(always)]
    pub fn from_account_info_init<'a>(account: &'a AccountView) -> Result<&'a mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(RawchunksError::InvalidInstructionData.into());
        }
        let data = unsafe { core::slice::from_raw_parts_mut(account.data_ptr() as *mut u8, account.data_len()) };
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

    pub fn find_pda(original_song_id: u64, remix_index: u64, program_id: &Address) -> (Address, u8) {
        let song_id_bytes = original_song_id.to_le_bytes();
        let index_bytes = remix_index.to_le_bytes();
        Address::derive_program_address(&[SEED_REMIX, &song_id_bytes, &index_bytes], program_id).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_struct_sizes_and_alignments() {
        assert_eq!(core::mem::size_of::<RoyaltiesConfig>(), RoyaltiesConfig::LEN);
        assert_eq!(RoyaltiesConfig::LEN % 8, 0);

        assert_eq!(core::mem::size_of::<FactoryConfig>(), FactoryConfig::LEN);
        assert_eq!(FactoryConfig::LEN % 8, 0);

        assert_eq!(core::mem::size_of::<SongAccount>(), SongAccount::LEN);
        assert_eq!(SongAccount::LEN % 8, 0);

        assert_eq!(core::mem::size_of::<AuthorizedConsumer>(), AuthorizedConsumer::LEN);
        assert_eq!(AuthorizedConsumer::LEN % 8, 0);

        assert_eq!(core::mem::size_of::<LicenseRecord>(), LicenseRecord::LEN);
        assert_eq!(LicenseRecord::LEN % 8, 0);

        assert_eq!(core::mem::size_of::<RemixConfig>(), RemixConfig::LEN);
        assert_eq!(RemixConfig::LEN % 8, 0);

        assert_eq!(core::mem::size_of::<SongRemixCount>(), SongRemixCount::LEN);
        assert_eq!(SongRemixCount::LEN % 8, 0);

        assert_eq!(core::mem::size_of::<RemixRecord>(), RemixRecord::LEN);
        assert_eq!(RemixRecord::LEN % 8, 0);
    }

    #[test]
    fn test_pda_derivations() {
        let dummy_program_id = Address::new_from_array([7u8; 32]);

        let (royalties_pda, r_bump) = RoyaltiesConfig::find_pda(&dummy_program_id);
        assert_ne!(royalties_pda.as_ref(), &[0u8; 32]);
        assert_eq!(r_bump, r_bump);

        let (factory_pda, f_bump) = FactoryConfig::find_pda(&dummy_program_id);
        assert_ne!(factory_pda.as_ref(), &[0u8; 32]);
        assert_eq!(f_bump, f_bump);

        let (song_pda, s_bump) = SongAccount::find_pda(1, &dummy_program_id);
        assert_ne!(song_pda.as_ref(), &[0u8; 32]);
        assert_eq!(s_bump, s_bump);

        let dummy_user = [9u8; 32];
        let (consumer_pda, c_bump) = AuthorizedConsumer::find_pda(&dummy_user, &dummy_program_id);
        assert_ne!(consumer_pda.as_ref(), &[0u8; 32]);
        assert_eq!(c_bump, c_bump);

        let (lic_pda, l_bump) = LicenseRecord::find_pda(1, &dummy_user, &dummy_program_id);
        assert_ne!(lic_pda.as_ref(), &[0u8; 32]);
        assert_eq!(l_bump, l_bump);

        let (remix_pda, rx_bump) = RemixConfig::find_pda(&dummy_program_id);
        assert_ne!(remix_pda.as_ref(), &[0u8; 32]);
        assert_eq!(rx_bump, rx_bump);

        let (auth_pda, a_bump) = RemixConfig::find_authority_pda(&dummy_program_id);
        assert_ne!(auth_pda.as_ref(), &[0u8; 32]);
        assert_eq!(a_bump, a_bump);

        let (cnt_pda, cnt_bump) = SongRemixCount::find_pda(1, &dummy_program_id);
        assert_ne!(cnt_pda.as_ref(), &[0u8; 32]);
        assert_eq!(cnt_bump, cnt_bump);

        let (rec_pda, rec_bump) = RemixRecord::find_pda(1, 0, &dummy_program_id);
        assert_ne!(rec_pda.as_ref(), &[0u8; 32]);
        assert_eq!(rec_bump, rec_bump);
    }
}
