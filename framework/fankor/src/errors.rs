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
#[error_code(offset = 0, skip_ts_gen)]
pub enum FankorErrorCode {
    // ------------------------------------------------------------------------
    // General ----------------------------------------------------------------
    // ------------------------------------------------------------------------
    /// The id of the program does not match the one defined in the code
    #[msg("The id of the program does not match the one defined in the code")]
    #[discriminant = 1000]
    DeclaredProgramIdMismatch,

    /// The instruction discriminant is missing
    #[msg("The instruction discriminant is missing")]
    MissingInstructionDiscriminant,

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

    /// The account to calculate the seeds is missing
    #[msg("The account to calculate the seeds is missing")]
    MissingSeedsAccount,

    /// The seeds of the PDA are missing
    #[msg("The seeds of the PDA ({}) are missing", account)]
    MissingPdaSeeds { account: Pubkey },

    // ------------------------------------------------------------------------
    // Accounts ---------------------------------------------------------------
    // ------------------------------------------------------------------------
    /// The instruction contains duplicated writable accounts
    #[msg("The instruction contains duplicated writable accounts: {}", address)]
    #[discriminant = 1500]
    DuplicatedWritableAccounts { address: Pubkey },

    /// The account discriminant did not match account's one
    #[msg("The account discriminant did not match account {}'s one", account)]
    AccountDiscriminantMismatch { account: String },

    /// Failed to deserialize the instruction account
    #[msg("Failed to deserialize the instruction account: {}", account)]
    InstructionDidNotDeserialize { account: String },

    /// Cannot modify an account that is not owned by the current program
    #[msg(
    "Cannot {} an account that is not owned by the current program: {}",
    action,
    address
    )]
    AccountNotOwnedByProgram {
        address: Pubkey,
        action: &'static str,
    },

    /// Cannot modify a readonly account
    #[msg("Cannot {} a readonly account: {}", action, address)]
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

    /// The account {} is not rent exempt
    #[msg("The account {} is not rent exempt", account)]
    AccountNotRentExempt { account: Pubkey },

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

    /// Sysvar account was expected to be correct
    #[msg("Sysvar account {} was expected to be {}", actual, expected)]
    IncorrectSysvarAccount { actual: Pubkey, expected: Pubkey },

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

    /// There is not enough data to deserialize the instruction
    #[msg("There is not enough data to deserialize the instruction")]
    NotEnoughDataToDeserializeInstruction,

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

    /// The constraint '{}' of the account '{}' has failed
    #[msg(
    "The constraint '{}' of the account '{}' has failed",
    constraint,
    account
    )]
    AccountConstraintFailed {
        account: &'static str,
        constraint: &'static str,
    },

    /// The specified account has different types.
    #[msg(
    "A duplicated account ({}) is deserialized with two different types",
    address
    )]
    DuplicatedAccountWithDifferentType { address: Pubkey },

    /// The account must be the default one.
    #[msg("The account must be the default one")]
    AccountNotDefault,

    // ------------------------------------------------------------------------
    // CPI --------------------------------------------------------------------
    // ------------------------------------------------------------------------
    /// The intermediate buffer is empty
    #[msg("The intermediate buffer is empty")]
    #[discriminant = 2000]
    EmptyIntermediateBuffer,

    /// The result of the intermediate buffer is expected to belong to one program but it belongs to another program instead
    #[msg(
    "The result of the intermediate buffer is expected to belong to the program {} but it belongs to the program {} instead ",
    expected,
    actual,
    )]
    IntermediateBufferIncorrectProgramId { actual: Pubkey, expected: Pubkey },

    /// The list contains too many accounts to correctly serialize the instruction. Max: 256
    #[msg(
    "The list contains too many accounts ({}) to correctly serialize the instruction. Max: 256",
    size
    )]
    TooManyAccounts { size: usize },

    // ------------------------------------------------------------------------
    // Zero Copy --------------------------------------------------------------
    // ------------------------------------------------------------------------
    /// Cannot deserialize the zero copy type
    #[msg("Cannot deserialize the zero copy type: '{}'", type_name)]
    #[discriminant = 2500]
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

    /// The value for a length field is too big.
    #[msg("The value for a length field is too big")]
    ZeroCopyLengthFieldOverflow,

    /// The provided field is not preceding the current one.
    #[msg("The provided field is not preceding the current one")]
    ZeroCopyIncorrectPrecedingField,

    /// Cannot move the specified bytes.
    #[msg("Cannot move the specified bytes")]
    ZeroCopyInvalidMove,
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
                    "ProgramError occurred. Error Name: {:?}. Error Code: {}. Error Message: {}.",
                    program_error,
                    u64::from(program_error.clone()),
                    program_error
                );
            }
            Error::FankorError(fankor_error) => fankor_error.log(),
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
            "FankorError occurred. Error Name: {}. Error Code: {}. Error Message: {}.",
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
