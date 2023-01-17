use crate::Result;
use quote::quote;
use syn::LitStr;

pub fn processor(pubkey: LitStr) -> Result<proc_macro::TokenStream> {
    let result = quote! {
        /// The static program ID.
        pub static ID: ::fankor::prelude::solana_program::pubkey::Pubkey = ::fankor::prelude::const_pubkey!(#pubkey);

        #[cfg(all(test, feature = "ts-gen"))]
        pub mod __ts_gen_test__setup {
            use ::fankor::prelude::ts_gen::BuildContext;
            use ::std::sync::Arc;

            // Helper to execute the tests in sequence.
            ::fankor::prelude::lazy_static! {
                pub static ref BUILD_CONTEXT: Arc<BuildContext> = Arc::new(BuildContext::new());
            }

            // ----------------------------------------------------------------
            // ----------------------------------------------------------------
            // ----------------------------------------------------------------

            /// This is the main build function.
            #[test]
            fn build() {
                BUILD_CONTEXT.build();
            }
        }
    };

    Ok(result.into())
}
