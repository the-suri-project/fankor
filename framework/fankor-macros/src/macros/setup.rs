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

        #[cfg(test)]
        #[test]
        fn __fankor_internal__test__id() { assert!(check_id(&id())); }

        // --------------------------------------------------------------------
        // --------------------------------------------------------------------
        // --------------------------------------------------------------------

        #[cfg(test)]
        pub mod __internal__idl_builder_test__root {
            ::fankor::test_helpers::lazy_static::lazy_static! {
                pub static ref ERROR_HELPER: ::fankor::test_helpers::ErrorHelper = ::fankor::test_helpers::ErrorHelper::new();
                pub static ref ACCOUNT_HELPER: ::fankor::test_helpers::AccountHelper = ::fankor::test_helpers::AccountHelper::new();
            }
        }
    };

    Ok(result.into())
}
