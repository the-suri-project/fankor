use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, Item, LitStr};

mod macros;
mod utils;

type Result<T> = std::result::Result<T, syn::Error>;

/// This macro setups the entry point of the framework.
#[proc_macro]
pub fn setup(args: TokenStream) -> TokenStream {
    let pubkey = parse_macro_input!(args as LitStr);

    match macros::setup::processor(pubkey) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// A custom implementation of BorshSerialize that fix an issue with the where clause.
#[proc_macro_derive(FankorSerialize, attributes(borsh_skip, discriminant))]
pub fn serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);

    match macros::serialize::processor(input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// A custom implementation of BorshDeserialize that fix an issue with the where clause.
#[proc_macro_derive(FankorDeserialize, attributes(borsh_skip, borsh_init, discriminant))]
pub fn deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);

    match macros::deserialize::processor(input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Implements the trait Account for the given struct.
#[proc_macro_derive(FankorZeroCopy)]
pub fn zero_copy(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);

    match macros::zero_copy::processor(input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// This macro marks defines a new account implementing the traits:
/// - `Accounts`
/// - `FankorSerialize`
/// - `FankorDeserialize`
#[proc_macro_attribute]
pub fn accounts(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);

    match macros::accounts::processor(args, input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// This macro marks defines a new account implementing the traits:
/// - `Account`
/// - `FankorSerialize`
/// - `FankorDeserialize`
#[proc_macro_attribute]
pub fn account(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);

    match macros::account::processor(args, input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Generates the size methods for structs and enums in order to get their minimum
/// size and the actual size.
#[proc_macro_derive(AccountSize)]
pub fn account_size(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);

    match macros::account_size::processor(input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Generates field offsets for structs and enums.
/// The offsets are based in the fixed (minimum) size of the accounts, so having:
/// ```ignore
/// struct Foo {
///     a: Vec<u8>,
///     b: u16,
/// }
/// ```
/// `a` will have offset 0 and `b` will have offset 4, because the minimum size of
/// a `Vec<_>` is just the 4-bytes length field with no content. This causes that
/// if `a` contains any value, the offset of `b` will be incorrect.
///
/// For those cases use `actual_offset` providing an object to get the correct offset
/// of a field inside that object.
///
/// > Requires that the struct or enum has the `AccountSize` trait implemented.
#[proc_macro_derive(AccountOffsets)]
pub fn account_offsets(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);

    match macros::account_offset::processor(input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Derives the `InstructionAccount` trait for the given struct/enum.
#[proc_macro_derive(InstructionAccounts, attributes(account, instruction))]
pub fn instruction_accounts(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);

    match macros::instruction_accounts::processor(input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// This macro defines an error list from an enum.
#[proc_macro_attribute]
pub fn error_code(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);

    match macros::error::processor(args, input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// This macro process a definition of a program.
/// A program consist of a set of instructions expressed as methods following one
/// of these signatures:
///
/// ```ignore
/// # Instruction without arguments
/// fn my_instruction(context: FankorContext, account: ACCOUNT) -> Result<RESULT>;
///
/// # Instruction with arguments
/// fn my_instruction(context: FankorContext, account: ACCOUNT, arguments: ARGS) -> Result<RESULT>;
///
/// # Fallback method
/// fn my_instruction(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> Result<RESULT>;
/// ```
///
/// Being `ACCOUNT` a type that implements the `InstructionAccount` trait and being `ARGS` and `RESULT` types
/// that implement the `FankorSerialize` and `FankorDeserialize` traits.
///
/// If `RESULT` is different from `()` then the instruction will store the result in the intermediate buffer as
/// the instruction result.
#[proc_macro_attribute]
pub fn program(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);

    match macros::program::processor(args, input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}
