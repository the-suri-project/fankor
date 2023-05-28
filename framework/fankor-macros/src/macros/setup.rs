use quote::quote;
use syn::LitStr;

use crate::Result;

pub fn processor(pubkey: LitStr) -> Result<proc_macro::TokenStream> {
    let result = quote! {
        /// The static program ID.
        #[::fankor::prelude::constant]
        pub const ID: ::fankor::prelude::solana_program::pubkey::Pubkey = ::fankor::prelude::const_pubkey!(#pubkey);

        #[cfg(feature = "ts-gen")]
        pub(crate) mod __ts_gen_test__setup {
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
