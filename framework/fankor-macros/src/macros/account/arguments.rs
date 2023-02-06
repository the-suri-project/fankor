use crate::fnk_syn::FnkMetaArgumentList;
use crate::Result;
use proc_macro2::Ident;

pub struct AccountArguments {
    /// The accounts type name.
    pub accounts_type_name: Ident,
}

impl AccountArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the AccountArguments struct from the given attributes.
    pub fn from(mut args: FnkMetaArgumentList) -> Result<AccountArguments> {
        args.error_on_duplicated()?;

        let result = AccountArguments {
            accounts_type_name: args.pop_ident("base", false)?.unwrap(),
        };

        args.error_on_unknown()?;

        Ok(result)
    }
}
