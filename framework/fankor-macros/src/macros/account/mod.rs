mod arguments;
mod ts_gen;

use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Item};

use crate::fnk_syn::FnkMetaArgumentList;
use crate::macros::account::arguments::AccountArguments;
use crate::macros::account::ts_gen::ts_gen;
use crate::Result;

pub fn processor(args: FnkMetaArgumentList, input: Item) -> Result<proc_macro::TokenStream> {
    // Process arguments.
    let arguments = AccountArguments::from(args)?;

    // Process input.
    let (name, generics, item) = match &input {
        Item::Struct(item) => (&item.ident, &item.generics, quote! { #item }),
        _ => {
            return Err(Error::new(
                input.span(),
                "account macro can only be applied to struct declarations",
            ));
        }
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let accounts_name = &arguments.accounts_type_name;
    let account_discriminants_name = format_ident!("{}Discriminant", accounts_name);
    let ts_gen = ts_gen(&input)?;

    let result = quote! {
        #[derive(FankorSerialize, FankorDeserialize, FankorZeroCopy, TsGen)]
        #[fankor(account = #account_discriminants_name)]
        #item

        #[automatically_derived]
        impl #impl_generics ::fankor::traits::AccountType for #name #ty_generics #where_clause {
             fn discriminant() -> u8 {
                #account_discriminants_name::#name.code()
            }

             fn owner() -> &'static Pubkey {
                &crate::ID
            }
        }

        #ts_gen
    };

    Ok(result.into())
}
