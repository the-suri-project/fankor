use fankor_syn::solana::parse_pubkey;
use fankor_syn::Result;
use quote::quote;
use syn::LitStr;

pub fn processor(pubkey: LitStr) -> Result<proc_macro::TokenStream> {
    let id = parse_pubkey(pubkey.value().as_str())?;

    let result = quote! {
        /// The static program ID.
        pub static ID: ::fankor::prelude::solana_program::pubkey::Pubkey = #id;
    };

    Ok(result.into())
}
