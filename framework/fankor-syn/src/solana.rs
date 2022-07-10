use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, LitByte};

use crate::Result;

/// Parses a textual `Pubkey` into the `Pubkey` constructor.
pub fn parse_pubkey(input: &str) -> Result<TokenStream> {
    let id_vec = match bs58::decode(input).into_vec() {
        Ok(v) => v,
        Err(_) => {
            return Err(Error::new(
                Span::call_site(),
                "Failed to decode base58 string".to_string(),
            ));
        }
    };

    let id_array = match <[u8; 32]>::try_from(<&[u8]>::clone(&&id_vec[..])) {
        Ok(v) => v,
        Err(_) => {
            return Err(Error::new(
                Span::call_site(),
                format!("Pubkey array is not 32 bytes long: len={}", id_vec.len()),
            ));
        }
    };

    let bytes = id_array.iter().map(|b| LitByte::new(*b, Span::call_site()));

    Ok(quote! {
        ::fankor::prelude::solana_program::pubkey::Pubkey::new_from_array(
            [#(#bytes,)*]
        )
    })
}
