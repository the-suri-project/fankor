use proc_macro2::Ident;

use crate::fnk_syn::FnkMetaArgumentList;
use crate::Result;

pub struct AccountsArguments {
    /// The main accounts type name. This helps to reuse the discriminants
    /// of the main accounts enum.
    pub accounts_type_name: Option<Ident>,
}

impl AccountsArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the AccountsArguments struct from the given attributes.
    pub fn from(mut args: FnkMetaArgumentList) -> Result<AccountsArguments> {
        args.error_on_duplicated()?;

        let result = AccountsArguments {
            accounts_type_name: args.pop_ident("base", true)?,
        };

        args.error_on_unknown()?;

        Ok(result)
    }
}
