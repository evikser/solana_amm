use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum AMMErrors {
    #[error("Invalid Instruction")]
    InvalidInstructionMethodID,

    #[error("Invalid Instruction")]
    InvalidInstructionData,

    #[error("Data account mismatch")]
    DataAccountMismatch,

    #[error("Token mint mismatch")]
    TokenMintMismatch,
}

impl From<AMMErrors> for ProgramError {
    fn from(e: AMMErrors) -> Self {
        ProgramError::Custom(e as u32)
    }
}
