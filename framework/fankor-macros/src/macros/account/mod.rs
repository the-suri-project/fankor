use fankor_syn::fankor::read_fankor_toml;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, Item};

use crate::utils::generate_discriminator;
use fankor_syn::Result;

pub fn processor(args: AttributeArgs, input: Item) -> Result<proc_macro::TokenStream> {
    // Process arguments.
    if !args.is_empty() {
        return Err(Error::new(
            input.span(),
            "account macro does not accept arguments",
        ));
    }

    // Read the Fankor.toml file.
    let config = read_fankor_toml();
    let accounts_config = config.accounts.as_ref().unwrap();

    // Process input.
    let result = match &input {
        Item::Struct(item) => {
            let name = &item.ident;
            let name_str = name.to_string();
            let generic_where_clause = &item.generics.where_clause;
            let generic_params = &item.generics.params;
            let generic_params = if generic_params.is_empty() {
                quote! {}
            } else {
                quote! { < #generic_params > }
            };

            let discriminator =
                if let Some(discriminator) = accounts_config.get_discriminator(&name_str) {
                    discriminator
                } else {
                    let discriminator = generate_discriminator(&name_str);
                    discriminator[..accounts_config.discriminator_size.unwrap() as usize].to_vec()
                };

            let test_unique_account_discriminator = format_ident!(
                "__fankor_internal__test__unique_account_discriminator_{}",
                name
            );

            let result = quote! {
                #[derive(FankorSerialize, FankorDeserialize)]
                #item

                #[automatically_derived]
                impl #generic_params ::fankor::traits::AccountSerialize for #name #generic_params #generic_where_clause {
                    fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> ::fankor::errors::FankorResult<()> {
                        if writer.write_all(<#name #generic_params as ::fankor::traits::Account>::discriminator()).is_err() {
                            return Err(::fankor::errors::ErrorCode::AccountDidNotSerialize{
                                account: #name_str.to_string()
                            }.into());
                        }

                        if ::fankor::prelude::borsh::BorshSerialize::serialize(self, writer).is_err() {
                            return Err(::fankor::errors::ErrorCode::AccountDidNotSerialize {
                                account: #name_str.to_string()
                            }.into());
                        }
                        Ok(())
                    }
                }

                #[automatically_derived]
                impl #generic_params ::fankor::traits::AccountDeserialize for #name #generic_params #generic_where_clause {
                    fn try_deserialize(buf: &mut &[u8]) -> ::fankor::errors::FankorResult<Self> {
                        let discriminator = <#name #generic_params as ::fankor::traits::Account>::discriminator();
                        let discriminator_len = discriminator.len();

                        if buf.len() < discriminator_len {
                            return Err(::fankor::errors::ErrorCode::AccountDiscriminatorNotFound{
                                account: #name_str.to_string()
                            }.into());
                        }

                        let given_disc = &buf[..discriminator_len];
                        if discriminator != given_disc {
                            return Err(::fankor::errors::ErrorCode::AccountDiscriminatorMismatch{
                                actual: given_disc.to_vec(),
                                expected: discriminator.to_vec(),
                                account: #name_str.to_string(),
                            }.into());
                        }
                        Self::try_deserialize_unchecked(buf)
                    }

                    fn try_deserialize_unchecked(buf: &mut &[u8]) -> ::fankor::errors::FankorResult<Self> {
                        let mut data: &[u8] = &buf[8..];
                        ::fankor::prelude::borsh::BorshDeserialize::deserialize(&mut data)
                            .map_err(|_| ::fankor::errors::ErrorCode::AccountDidNotDeserialize {
                            account: #name_str.to_string()
                        }.into())
                    }
                }

                #[automatically_derived]
                impl #generic_params ::fankor::traits::Account for #name #generic_params #generic_where_clause {
                     fn discriminator() -> &'static [u8] {
                        &[#(#discriminator,)*]
                    }

                     fn owner() -> Pubkey {
                        crate::ID
                    }
                }

                #[allow(non_snake_case)]
                #[automatically_derived]
                #[cfg(test)]
                #[test]
                fn #test_unique_account_discriminator() {
                    let account_name = #name_str;
                    let discriminator = <#name #generic_params as ::fankor::traits::Account>::discriminator();
                    let helper = &crate::__internal__idl_builder_test__root::ACCOUNT_HELPER;

                    if let Err(item) = helper.add_error(account_name, discriminator) {
                        panic!("There is a discriminator collision between accounts. First: {}, Second: {}, Discriminator: {:?}", account_name, item.account_name, discriminator);
                    }
                }
            };

            result
        }
        Item::Enum(_v) => {
            unimplemented!("b")
        }
        _ => {
            return Err(Error::new(
                input.span(),
                "account macro can only be applied to struct or enum declarations",
            ));
        }
    };

    // TODO add test to ensure that the discriminator is unique.

    Ok(result.into())
}
