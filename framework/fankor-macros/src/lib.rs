extern crate core;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, Item};

use fankor_syn::Result;

mod macros;
mod utils;

/// This macro setups the entry point of the framework.
#[proc_macro]
pub fn setup(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);

    assert!(args.is_empty(), "setup macro takes no arguments");

    match macros::setup::processor() {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// A custom implementation of BorshSerialize that fix an issue with the where clause.
#[proc_macro_derive(FankorSerialize, attributes(borsh_skip))]
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
#[proc_macro_derive(FankorDeserialize, attributes(borsh_skip, borsh_init))]
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

/// This macro marks defines a new account implementing the traits:
/// - `Account`
/// - `AccountSerialize`
/// - `AccountDeserialize`
/// - `BorshSerialize`
/// - `BorshDeserialize`
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

    match macros::account::size::processor(input) {
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

    match macros::account::offset::processor(input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// This macro transforms an enum into an error enum.
#[proc_macro_attribute]
pub fn error_code(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);

    match macros::error::processor(args, input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}
