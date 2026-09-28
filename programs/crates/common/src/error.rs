use pinocchio::error::ProgramError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum RawchunksError {
    NotAdmin = 6000,
    InvalidAddress = 6001,
    SongNotFound = 6002,
    NotSongCreator = 6003,
    NotAuthorized = 6004,
    EmptyMetadataUri = 6005,
    SongNotRemixable = 6006,
    RemixCannotBeRemixed = 6007,
    NoLicensesAvailable = 6008,
    UsdcNotSet = 6009,
    AlreadyInitialized = 6010,
    InvalidSeeds = 6011,
    MathOverflow = 6012,
    AccountNotWritable = 6013,
    MissingRequiredSignature = 6014,
    InvalidAccountOwner = 6015,
    InvalidInstructionData = 6016,
    IncorrectProgramId = 6017,
}

impl From<RawchunksError> for ProgramError {
    fn from(e: RawchunksError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<RawchunksError> for u32 {
    fn from(e: RawchunksError) -> Self {
        e as u32
    }
}
