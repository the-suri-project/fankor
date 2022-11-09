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

        // --------------------------------------------------------------------
        // --------------------------------------------------------------------
        // --------------------------------------------------------------------

        #[cfg(test)]
        pub mod __internal__idl_builder_test__root {
            ::fankor::test_helpers::lazy_static::lazy_static! {
                pub static ref INSTRUCTION_HELPER: ::fankor::test_helpers::InstructionHelper = ::fankor::test_helpers::InstructionHelper::new();
                pub static ref PROGRAM_HELPER: ::fankor::test_helpers::ProgramHelper = ::fankor::test_helpers::ProgramHelper::new();
            }
        }
    };

    Ok(result.into())
}
