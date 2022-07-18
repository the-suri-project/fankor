use syn::spanned::Spanned;
use syn::{Error, Item};

use fankor_syn::Result;

use crate::macros::instruction_accounts::r#enum::process_enum;
use crate::macros::instruction_accounts::r#struct::process_struct;

mod r#enum;
mod field;
mod parser;
mod r#struct;

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
