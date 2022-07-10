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
    match macros::deserialize::processor(input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// This macro marks a constant to be exposed into the IDL.
#[proc_macro_attribute]
pub fn constant(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);

    match macros::constant::processor(args, input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}
