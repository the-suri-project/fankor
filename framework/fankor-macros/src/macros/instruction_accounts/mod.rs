use syn::spanned::Spanned;
use syn::{Error, Item};

use crate::Result;

use crate::macros::instruction_accounts::enums::process_enum;
use crate::macros::instruction_accounts::structs::process_struct;

mod arguments;
mod enums;
mod field;
mod parser;
mod structs;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    // Process input.
    let result = match input {
        Item::Struct(item) => process_struct(item)?,
        Item::Enum(item) => process_enum(item)?,
        _ => {
            return Err(Error::new(
                input.span(),
                "Instruction macro can only be applied to struct or enum declarations",
            ));
        }
    };

    Ok(result)
}
