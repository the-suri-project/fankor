pub type FankorResult<T> = Result<T, Error>;

use fankor_macros::error_code;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use std::fmt::{Debug, Display};

use crate as fankor;

/// The starting point for user defined error codes.
pub const ERROR_CODE_OFFSET: u32 = 6000;

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    FankorError(FankorError),
    ProgramError(ProgramError),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FankorError(e) => Display::fmt(&e, f),
            Error::ProgramError(e) => Display::fmt(&e, f),
        }
    }
}

impl From<FankorError> for Error {
    fn from(e: FankorError) -> Self {
        Self::FankorError(e)
    }
}

impl From<ProgramError> for Error {
    fn from(e: ProgramError) -> Self {
        Self::ProgramError(e)
    }
}

impl From<borsh::maybestd::io::Error> for Error {
    fn from(e: borsh::maybestd::io::Error) -> Self {
        Error::ProgramError(ProgramError::from(e))
    }
}

impl From<&dyn std::error::Error> for Error {
    fn from(e: &dyn std::error::Error) -> Self {
        Error::FankorError(FankorError {
            error_name: "Unknown error".to_string(),
            error_msg: e.to_string(),
            error_code_number: 5500,
        })
    }
}

impl Error {
    pub fn log(&self) {
        match self {
            Error::ProgramError(program_error) => {
                msg!(
                    "ProgramError occurred. Error Code: {:?}. Error Number: {}. Error Message: {}.",
                    program_error,
                    u64::from(program_error.clone()),
                    program_error
                );
            }
            Error::FankorError(anchor_error) => anchor_error.log(),
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct FankorError {
    pub error_name: String,
    pub error_code_number: u32,
    pub error_msg: String,
}

impl FankorError {
    pub fn log(&self) {
        msg!(
            "FankorError occurred. Error Code: {}. Error Number: {}. Error Message: {}.",
            self.error_name,
            self.error_code_number,
            self.error_msg
        );
    }
}

impl Display for FankorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

/// Two `FankorError`s are equal when they have the same error code
impl PartialEq for FankorError {
    fn eq(&self, other: &Self) -> bool {
        self.error_code_number == other.error_code_number
    }
}

impl Eq for FankorError {}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl From<Error> for ProgramError {
    fn from(e: Error) -> ProgramError {
        match e {
            Error::FankorError(FankorError {
                error_code_number, ..
            }) => ProgramError::Custom(error_code_number),
            Error::ProgramError(program_error) => program_error,
        }
    }
}
