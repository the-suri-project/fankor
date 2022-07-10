extern crate core;

use syn::{parse_macro_input, AttributeArgs, Item};

use fankor_syn::Result;

mod macros;
mod utils;

/// This macro setups the entry point of the framework.
#[proc_macro]
pub fn setup(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
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

/// This macro marks a constant to be exposed into the IDL.
#[proc_macro_attribute]
pub fn constant(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);

    match macros::constant::processor(args, input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}
