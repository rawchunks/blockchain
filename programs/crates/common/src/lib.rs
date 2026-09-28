#![no_std]

pub mod error;
pub mod state;

pub use error::RawchunksError;
pub use state::*;

// Constants
pub const BASIS_POINTS: u64 = 10_000;
pub const CREATOR_SHARE_BPS: u64 = 8_000;  // 80%
pub const TREASURY_SHARE_BPS: u64 = 2_000; // 20%
pub const TREASURY_FEE_BPS: u64 = TREASURY_SHARE_BPS; // Alias for consistency

pub const MAX_METADATA_URI_LEN: usize = 200;

// PDA Seeds
pub const SEED_ROYALTIES: &[u8] = b"royalties";
pub const SEED_FACTORY: &[u8] = b"factory";
pub const SEED_SONG: &[u8] = b"song";
pub const SEED_CONSUMER: &[u8] = b"consumer";
pub const SEED_LICENSE: &[u8] = b"license";
pub const SEED_REMIX: &[u8] = b"remix";
pub const SEED_REMIX_AUTHORITY: &[u8] = b"remix_authority";
pub const SEED_REMIX_COUNT: &[u8] = b"remix_count";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shares_sum_to_basis_points() {
        assert_eq!(CREATOR_SHARE_BPS + TREASURY_SHARE_BPS, BASIS_POINTS);
    }

    #[test]
    fn test_split_calculation() {
        let license_price: u64 = 100_000_000; // 100 USDC (6 decimals)
        let creator_share = (license_price as u128)
            .checked_mul(CREATOR_SHARE_BPS as u128)
            .unwrap()
            .checked_div(BASIS_POINTS as u128)
            .unwrap() as u64;
        let treasury_share = license_price.checked_sub(creator_share).unwrap();

        assert_eq!(creator_share, 80_000_000);
        assert_eq!(treasury_share, 20_000_000);
        assert_eq!(creator_share + treasury_share, license_price);
    }

    #[test]
    fn test_split_calculation_odd_number() {
        let license_price: u64 = 10_000_001;
        let creator_share = (license_price as u128)
            .checked_mul(CREATOR_SHARE_BPS as u128)
            .unwrap()
            .checked_div(BASIS_POINTS as u128)
            .unwrap() as u64;
        let treasury_share = license_price.checked_sub(creator_share).unwrap();

        assert_eq!(creator_share + treasury_share, license_price);
        assert_eq!(creator_share, 8_000_000);
        assert_eq!(treasury_share, 2_000_001);
    }

    #[test]
    fn test_error_conversions() {
        let err = RawchunksError::SongNotRemixable;
        let code: u32 = err.into();
        assert_eq!(code, 6006);
    }
}
