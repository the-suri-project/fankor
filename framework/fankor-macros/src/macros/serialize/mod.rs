use proc_macro2::Span;
use quote::quote;
use syn::{Error, Ident, Item, ItemImpl};
use syn::spanned::Spanned;

use crate::macros::serialize::enums::enum_ser;
use crate::macros::serialize::structs::struct_ser;
use crate::Result;

mod enums;
mod structs;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    let crate_name = Ident::new("borsh", Span::call_site());

    // Process input.
    let (res, initial_where_clause) = match input {
        Item::Struct(input) => {
            let initial_where_clause = input.generics.where_clause.clone();

            (struct_ser(&input, crate_name)?, initial_where_clause)
        }
        Item::Enum(input) => {
            let initial_where_clause = input.generics.where_clause.clone();

            (enum_ser(&input, crate_name)?, initial_where_clause)
        }
        _ => {
            return Err(Error::new(
                input.span(),
                "FankorSerialize macro can only be applied to struct or enum declarations",
            ));
        }
    };

    let mut impl_block = syn::parse2::<ItemImpl>(res).unwrap();
    impl_block.generics.where_clause = initial_where_clause;

    Ok(proc_macro::TokenStream::from(quote! {
        #impl_block
    }))
}
