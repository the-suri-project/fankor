use crate::Result;
use quote::quote;
use syn::LitStr;

pub fn processor(pubkey: LitStr) -> Result<proc_macro::TokenStream> {
    let result = quote! {
        /// The static program ID.
        pub static ID: ::fankor::prelude::solana_program::pubkey::Pubkey = ::fankor::prelude::const_pubkey!(#pubkey);
    };

    Ok(result.into())
}
