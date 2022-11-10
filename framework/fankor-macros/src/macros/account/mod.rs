mod arguments;

use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, Item};

use crate::macros::account::arguments::AccountArguments;
use fankor_syn::Result;

pub fn processor(args: AttributeArgs, input: Item) -> Result<proc_macro::TokenStream> {
    // Process arguments.
    let arguments = AccountArguments::from(args, input.span())?;

    // Process input.
    let (name, generics, item) = match &input {
        Item::Struct(item) => (&item.ident, &item.generics, quote! { #item }),
        Item::Enum(item) => (&item.ident, &item.generics, quote! { #item }),
        _ => {
            return Err(Error::new(
                input.span(),
                "account macro can only be applied to struct or enum declarations",
            ));
        }
    };

    let name_str = name.to_string();
    let accounts_name = &arguments.accounts_type_name;
    let account_discriminants_name = format_ident!("{}Discriminant", accounts_name);
    let generic_where_clause = &generics.where_clause;
    let generic_params = &generics.params;
    let generic_params = if generic_params.is_empty() {
        quote! {}
    } else {
        quote! { < #generic_params > }
    };

    let result = quote! {
        #[derive(FankorSerialize, FankorDeserialize)]
        #item

        #[automatically_derived]
        impl #generic_params ::fankor::traits::AccountSerialize for #name #generic_params #generic_where_clause {
            fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> ::fankor::errors::FankorResult<()> {
                if writer.write_all(&[<#name #generic_params as ::fankor::traits::Account>::discriminator()]).is_err() {
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
        impl #generic_params ::fankor::traits::AccountDeserialize for #name #generic_params #generic_where_clause {
            fn try_deserialize(buf: &mut &[u8]) -> ::fankor::errors::FankorResult<Self> {
                let account: #accounts_name = ::fankor::prelude::borsh::BorshDeserialize::deserialize(buf)
                    .map_err(|_| ::fankor::errors::FankorErrorCode::AccountDiscriminatorNotFound {
                    account: #name_str.to_string()
                })?;

                let account = match account {
                    #accounts_name::#name(v) => v,
                    _ => return Err(
                        ::fankor::errors::FankorErrorCode::AccountDiscriminatorMismatch {
                            account: #name_str.to_string(),
                            expected: <#name #generic_params as ::fankor::traits::Account>::discriminator(),
                            actual: account.discriminant_code()
                        }
                        .into(),
                    )
                };

                Ok(account)
            }

            unsafe fn try_deserialize_unchecked(buf: &mut &[u8]) -> ::fankor::errors::FankorResult<Self> {
                ::fankor::prelude::borsh::BorshDeserialize::deserialize(buf)
                    .map_err(|_| ::fankor::errors::FankorErrorCode::AccountDidNotDeserialize {
                    account: #name_str.to_string()
                }.into())
            }
        }

        #[automatically_derived]
        impl #generic_params ::fankor::traits::Account for #name #generic_params #generic_where_clause {
             fn discriminator() -> u8 {
                #account_discriminants_name::#name.code()
            }

             fn owner() -> &'static Pubkey {
                &crate::ID
            }
        }
    };

    Ok(result.into())
}
