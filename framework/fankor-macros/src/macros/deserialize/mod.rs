use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Error, Ident, Item};

use crate::macros::deserialize::enums::enum_de;
use crate::macros::deserialize::structs::struct_de;
use crate::Result;

mod enums;
mod structs;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    let crate_name = Ident::new("borsh", Span::call_site());

    // Process input.
    let result = match input {
        Item::Struct(input) => struct_de(&input, crate_name)?,
        Item::Enum(input) => enum_de(&input, crate_name)?,
        _ => {
            return Err(Error::new(
                input.span(),
                "FankorSerialize macro can only be applied to struct or enum declarations",
            ));
        }
    };

    Ok(result.into())
}
