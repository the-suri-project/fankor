use fankor_syn::fankor::read_fankor_toml;
use fankor_syn::solana::parse_pubkey;
use fankor_syn::Result;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs};

/// This macro setups the entry point of the framework.
#[proc_macro]
pub fn setup(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);

    assert!(args.is_empty(), "setup macro takes no arguments");

    match processor() {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

fn processor() -> Result<proc_macro::TokenStream> {
    // Read the Fankor.toml file.
    let config = read_fankor_toml();
    let id = parse_pubkey(config.program.pubkey.as_str())?;

    let result = quote! {
        /// The static program ID
        pub static ID: ::fankor::prelude::solana_program::pubkey::Pubkey = #id;

        // --------------------------------------------------------------------
        // --------------------------------------------------------------------
        // --------------------------------------------------------------------

        #[cfg(all(test, feature = "builder-tests"))]
        pub mod __internal__idl_builder_tests__root {
            use super::ID;

            ::fankor::build::prelude::lazy_static! {
                pub static ref IDL_CONTEXT: ::std::sync::Arc<::fankor::build::IdlContext> = ::std::sync::Arc::new(::fankor::build::IdlContext::new());
            }

            // ----------------------------------------------------------------
            // ----------------------------------------------------------------
            // ----------------------------------------------------------------

            /// This is the main build function.
            #[test]
            fn build() {
                IDL_CONTEXT.build();
            }
        }
    };

    Ok(result.into())
}
