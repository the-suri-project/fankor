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

    let name_str = name.to_string();
    let accounts_name = &arguments.accounts_type_name;
    let account_discriminants_name = format_ident!("{}Discriminant", accounts_name);
    let ts_gen = ts_gen(&input, &account_discriminants_name)?;

    let result = quote! {
        #[derive(FankorSerialize, FankorDeserialize)]
        #item

        #[automatically_derived]
        impl #impl_generics ::fankor::traits::AccountSerialize for #name #ty_generics #where_clause {
            fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> ::fankor::errors::FankorResult<()> {
                if writer.write_all(&[<#name #ty_generics as ::fankor::traits::AccountType>::discriminant()]).is_err() {
                    return Err(::fankor::errors::FankorErrorCode::AccountDidNotSerialize{
                        account: #name_str.to_string()
                    }.into());
                }

                if ::fankor::prelude::borsh::BorshSerialize::serialize(self, writer).is_err() {
                    return Err(::fankor::errors::FankorErrorCode::AccountDidNotSerialize {
                        account: #name_str.to_string()
                    }.into());
                }

                Ok(())
            }
        }

        #[automatically_derived]
        impl #impl_generics ::fankor::traits::AccountDeserialize for #name #ty_generics #where_clause {
            fn try_deserialize(buf: &mut &[u8]) -> ::fankor::errors::FankorResult<Self> {
                let account: #accounts_name = ::fankor::prelude::borsh::BorshDeserialize::deserialize(buf)
                    .map_err(|_| ::fankor::errors::FankorErrorCode::AccountDiscriminantNotFound {
                    account: #name_str.to_string()
                })?;

                let account = match account {
                    #accounts_name::#name(v) => v,
                    _ => return Err(
                        ::fankor::errors::FankorErrorCode::AccountDiscriminantMismatch {
                            account: #name_str.to_string(),
                        }
                        .into(),
                    )
                };

                Ok(account)
            }

            fn try_deserialize_unchecked(buf: &mut &[u8]) -> ::fankor::errors::FankorResult<Self> {
                ::fankor::prelude::borsh::BorshDeserialize::deserialize(buf)
                    .map_err(|_| ::fankor::errors::FankorErrorCode::AccountDidNotDeserialize {
                    account: #name_str.to_string()
                }.into())
            }
        }

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
