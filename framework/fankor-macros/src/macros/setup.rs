use fankor_syn::fankor::read_fankor_toml;
use fankor_syn::solana::parse_pubkey;
use fankor_syn::Result;
use quote::quote;

pub fn processor() -> Result<proc_macro::TokenStream> {
    // Read the Fankor.toml file.
    let config = read_fankor_toml();
    let id = parse_pubkey(config.program.pubkey.as_str())?;

    let result = quote! {
        /// The static program ID.
        pub static ID: ::fankor::prelude::solana_program::pubkey::Pubkey = #id;

        /// Confirms that a given pubkey is equivalent to the program ID.
        pub fn check_id(id: &::fankor::prelude::solana_program::pubkey::Pubkey) -> bool { id == &ID }

        /// Returns the program ID.
        pub fn id() -> ::fankor::prelude::solana_program::pubkey::Pubkey { ID }

        #[test]
        fn test_id() { assert!(check_id(&id())); }

        // --------------------------------------------------------------------
        // --------------------------------------------------------------------
        // --------------------------------------------------------------------

        #[cfg(all(test, feature = "builder"))]
        pub mod __internal__idl_builder_test__root {
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
