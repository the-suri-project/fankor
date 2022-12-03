use std::fmt::{Debug, Display};

use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use fankor_macros::error_code;

use crate as fankor;

pub type FankorResult<T> = Result<T, Error>;

/// The starting point for user defined error codes.
pub const ERROR_CODE_OFFSET: u32 = 6000;

/// Error codes that can be returned by internal framework code.
///
/// - 1000..1499 - General
/// - 1500..1999 - Accounts
/// - 2000..2499 - CPI
/// - 2500..2999 - ZeroCopy
///
/// The starting point for user-defined errors is defined
/// by the [ERROR_CODE_OFFSET](crate::error::ERROR_CODE_OFFSET).
#[error_code(offset = 0)]
pub enum FankorErrorCode {
    // ------------------------------------------------------------------------
    // General ----------------------------------------------------------------
    // ------------------------------------------------------------------------
    /// The id of the program does not match the one defined in the code
    #[msg("The id of the program does not match the one defined in the code")]
    #[code(1000)]
    DeclaredProgramIdMismatch,

    /// The instruction discriminant is missing
    #[msg("The instruction discriminant is missing")]
    InstructionDiscriminantMissing,

    /// The instruction discriminant did not match any valid
    #[msg("The instruction discriminant did not match any valid")]
    InstructionDiscriminantNotFound,

    /// The instruction contains more accounts than required
    #[msg("The instruction contains more accounts than required")]
    UnusedAccounts,

    /// The program must be provided in the account list
    #[msg(
        "The program {} ({}) must be provided in the account list",
        name,
        address
    )]
    MissingProgram { address: Pubkey, name: &'static str },

    /// Cannot find a valid PDA with the provided seeds for the specified program
    #[msg(
        "Cannot find a valid PDA with the provided seeds for the specified program: {}",
        program_id
    )]
    CannotFindValidPdaWithProvidedSeeds { program_id: Pubkey },

    /// The provided PDA does not match expected one
    #[msg(
        "The provided PDA ({}) does not match expected one ({})",
        actual,
        expected
    )]
    InvalidPda { expected: Pubkey, actual: Pubkey },

    /// The ump seed of the PDA is missing
    #[msg("The ump seed of the PDA ({}) is missing", account)]
    MissingPdaBumpSeed { account: Pubkey },

    // ------------------------------------------------------------------------
    // Accounts ---------------------------------------------------------------
    // ------------------------------------------------------------------------
    /// The instruction contains duplicated mutable accounts
    #[msg("The instruction contains duplicated mutable accounts: {}", address)]
    #[code(1500)]
    DuplicatedMutableAccounts { address: Pubkey },

    /// No 8 byte discriminant was found on the account
    #[msg("No 8 byte discriminant was found on the account: {}", account)]
    AccountDiscriminantNotFound { account: String },

    /// The account discriminant did not match what was expected
    #[msg(
        "The account discriminant {} did not match what was expected {} of account {}",
        actual,
        expected,
        account
    )]
    AccountDiscriminantMismatch {
        actual: u8,
        expected: u8,
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

    /// The instruction expects no accounts
    #[msg("The instruction expects no accounts")]
    NotAccountsExpected,

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

    /// The length of the account list must be greater or equal than one value but it is another
    #[msg(
        "The length of the account list '{}' must be greater or equal than {} but it is {}",
        account,
        expected,
        actual
    )]
    AccountConstraintMinimumMismatch {
        actual: usize,
        expected: usize,
        account: &'static str,
    },

    /// The length of the account list must be lower or equal than one value but it is another
    #[msg(
        "The length of the account list '{}' must be lower or equal than {} but it is {}",
        account,
        expected,
        actual
    )]
    AccountConstraintMaximumMismatch {
        actual: usize,
        expected: usize,
        account: &'static str,
    },

    // ------------------------------------------------------------------------
    // CPI --------------------------------------------------------------------
    // ------------------------------------------------------------------------
    /// The intermediate buffer is empty
    #[msg("The intermediate buffer is empty")]
    #[code(2000)]
    EmptyIntermediateBuffer,

    /// The result of the intermediate buffer is expected to belong to one program but it belongs to another program instead
    #[msg(
        "The result of the intermediate buffer is expected to belong to the program {} but it belongs to the program {} instead ",
        expected,
        actual,
    )]
    IntermediateBufferIncorrectProgramId { actual: Pubkey, expected: Pubkey },

    // ------------------------------------------------------------------------
    // Zero Copy --------------------------------------------------------------
    // ------------------------------------------------------------------------
    /// Cannot deserialize the zero copy type
    #[msg("Cannot deserialize the zero copy type: '{}'", type_name)]
    #[code(2500)]
    ZeroCopyCannotDeserialize { type_name: &'static str },

    /// Not enough length to deserialize the zero copy type
    #[msg("Not enough length to deserialize the zero copy type: '{}'", type_name)]
    ZeroCopyNotEnoughLength { type_name: &'static str },

    /// Invalid enum discriminant while deserializing the zero copy type
    #[msg(
        "Invalid enum discriminant while deserializing the zero copy type: '{}'",
        type_name
    )]
    ZeroCopyInvalidEnumDiscriminant { type_name: &'static str },

    /// Possible deadlock trying to access a zero copy type
    #[msg("Possible deadlock trying to access a zero copy type: '{}'", type_name)]
    ZeroCopyPossibleDeadlock { type_name: &'static str },
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
