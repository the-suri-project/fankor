use syn::spanned::Spanned;
use syn::{Error, Item};

use crate::fnk_syn::FnkMetaArgumentList;
use crate::macros::instruction::enums::process_enum;
use crate::macros::instruction::structs::process_struct;
use crate::Result;

mod arguments;
mod enums;
mod field;
mod parser;
mod structs;

pub fn processor(args: FnkMetaArgumentList, input: Item) -> Result<proc_macro::TokenStream> {
    // Process input.
    let result = match input {
        Item::Struct(item) => process_struct(args, item)?,
        Item::Enum(item) => process_enum(args, item)?,
        _ => {
            return Err(Error::new(
                input.span(),
                "Instruction macro can only be applied to struct or enum declarations",
            ));
        }
    };

    Ok(result)
}
