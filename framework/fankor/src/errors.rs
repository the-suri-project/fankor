pub type FankorResult<T> = Result<T, Error>;

use fankor_macros::error_code;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::fmt::{Debug, Display};

use crate as fankor;

/// The starting point for user defined error codes.
pub const ERROR_CODE_OFFSET: u32 = 6000;

/// Error codes that can be returned by internal framework code.
///
/// - 3000..4000 - Accounts
/// - 5500       - custom program error without code
///
/// The starting point for user-defined errors is defined
/// by the [ERROR_CODE_OFFSET](crate::error::ERROR_CODE_OFFSET).
#[error_code(offset = 0)]
pub enum ErrorCode {
    // Accounts
    /// No 8 byte discriminator was found on the account
    #[msg("No 8 byte discriminator was found on the account: {}", account)]
    #[continue_from(3000)]
    AccountDiscriminatorNotFound { account: String },

    /// 8 byte discriminator did not match what was expected
    #[msg(
        "8 byte discriminator {:?} did not match what was expected {:?} of account {}",
        actual,
        expected,
        account
    )]
    AccountDiscriminatorMismatch {
        actual: Vec<u8>,
        expected: Vec<u8>,
        account: String,
    },

    /// Failed to serialize the account
    #[msg("Failed to serialize the account: {}", account)]
    AccountDidNotSerialize { account: String },

    /// Failed to deserialize the account
    #[msg("Failed to deserialize the account: {}", account)]
    AccountDidNotDeserialize { account: String },

    /// Cannot modify an account that is not owned by the current program
    #[msg(
        "Cannot {} an account that is not owned by the current program: {}",
        address,
        action
    )]
    AccountNotOwnedByProgram {
        address: Pubkey,
        action: &'static str,
    },

    /// Cannot modify a readonly account
    #[msg("Cannot {} a readonly account: {}", address, action)]
    ReadonlyAccountModification {
        address: Pubkey,
        action: &'static str,
    },

    /// Cannot create a mutable reference to a readonly account
    #[msg("Cannot create a mutable reference to a readonly account: {}", address)]
    MutRefToReadonlyAccount { address: Pubkey },

    /// Cannot create an account from an AccountInfo which has been already marked as closed. If your purpose is to revive the account, please use: FankorContext::revive
    #[msg("Cannot create an account from an AccountInfo ({}) which has been already marked as closed. If your purpose is to revive the account, please use: FankorContext::revive", address)]
    NewFromClosedAccount { address: Pubkey },

    /// Cannot modify a readonly account
    #[msg("Cannot {} a readonly account: {}", address, action)]
    AccountNotRentExempt {
        address: Pubkey,
        action: &'static str,
    },

    /// Account not initialized
    #[msg("Account {} not initialized", address)]
    AccountNotInitialized { address: Pubkey },

    /// The account is already initialized
    #[msg("The account {} is already initialized", address)]
    AccountAlreadyInitialized { address: Pubkey },

    /// Account was expected to be owned by a program but it is owned by another
    #[msg(
        "Account {} was expected to be owned by program {} but it is owned by {}",
        address,
        expected,
        actual
    )]
    AccountOwnedByWrongProgram {
        address: Pubkey,
        expected: Pubkey,
        actual: Pubkey,
    },

    /// The account cannot be writen because it is already closed
    #[msg(
        "Cannot {} the account {} because it is already closed",
        action,
        address
    )]
    AlreadyClosedAccount {
        address: Pubkey,
        action: &'static str,
    },

    /// A program was expected but it is another instead
    #[msg("The program {} was expected but it is {} instead", expected, actual)]
    InvalidProgram { expected: Pubkey, actual: Pubkey },

    /// The program was expected to be executable
    #[msg("The program {} was expected to be executable", program)]
    ProgramIsNotExecutable { program: Pubkey },

    /// There are not enough accounts to deserialize the instruction
    #[msg("There are not enough accounts to deserialize the instruction")]
    NotEnoughAccountKeys,

    /// There are not enough valid accounts to deserialize the account list
    #[msg("There are not enough valid accounts to deserialize the account list")]
    NotEnoughValidAccountForVec,

    /// The account must belong to a program but it belongs to another
    #[msg(
        "The account '{}' must belong to program {} but it belongs to {}",
        account,
        expected,
        actual
    )]
    AccountConstraintOwnerMismatch {
        actual: Pubkey,
        expected: Pubkey,
        account: &'static str,
    },

    /// The account's address of an account must be one value but it is another
    #[msg(
        "The account's address of '{}' must be {} but it is {}",
        account,
        expected,
        actual
    )]
    AccountConstraintAddressMismatch {
        actual: Pubkey,
        expected: Pubkey,
        account: &'static str,
    },

    /// The account must be initialized
    #[msg("The account '{}' must be initialized", account)]
    AccountConstraintNotInitialized { account: &'static str },

    /// The account must not be initialized
    #[msg("The account '{}' must not be initialized", account)]
    AccountConstraintInitialized { account: &'static str },

    /// The account must be writable
    #[msg("The account '{}' must be writable", account)]
    AccountConstraintNotWritable { account: &'static str },

    /// The account must not be writable
    #[msg("The account '{}' must not be writable", account)]
    AccountConstraintWritable { account: &'static str },

    /// The account must be executable
    #[msg("The account '{}' must be executable", account)]
    AccountConstraintNotExecutable { account: &'static str },

    /// The account must not be executable
    #[msg("The account '{}' must not be executable", account)]
    AccountConstraintExecutable { account: &'static str },

    /// The account must be a rent-exempt
    #[msg("The account '{}' must be a rent-exempt", account)]
    AccountConstraintNotRentExempt { account: &'static str },

    /// The account must not be rent-exempt
    #[msg("The account '{}' must not be a rent-exempt", account)]
    AccountConstraintRentExempt { account: &'static str },

    /// The account must be a signer
    #[msg("The account '{}' must be a signer", account)]
    AccountConstraintNotSigner { account: &'static str },

    /// The account must not be a signer
    #[msg("The account '{}' must not be a signer", account)]
    AccountConstraintSigner { account: &'static str },

    /// The length of the account list must be one value but it is another
    #[msg(
        "The length of the account list '{}' must be {} but it is {}",
        account,
        expected,
        actual
    )]
    AccountConstraintMinimumMismatch {
        actual: usize,
        expected: usize,
        account: &'static str,
    },
}

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
