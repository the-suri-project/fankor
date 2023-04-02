use crate::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Item;

pub fn ts_gen(input: &Item) -> Result<TokenStream> {
    // Process input.
    let name = match &input {
        Item::Struct(item) => &item.ident,
        Item::Enum(item) => &item.ident,
        _ => unreachable!(),
    };

    let name_str = name.to_string();

    let type_extension = format!(
        "export namespace {} {{
            export async function fetchAccountByAddress(
                connection: solana.Connection,
                address: solana.PublicKey
            ): Promise<fnk.AccountResult<{}> | null> {{
                let account = await connection.getAccountInfo(
                    address,
                    connection.commitment
                );

                if (account) {{
                    if (fnk.equals(ID, account.owner)) {{
                        let buf = account.data;
                        let data = this.deserialize(buf);
                        return {{
                            address,
                            account,
                            data,
                        }};
                    }}
                }}

                return null;
            }}

            export async function fetchManyAccountsByAddress(connection: solana.Connection,
                addresses: solana.PublicKey[]): Promise<(fnk.AccountResult<{}> | null)[]> {{
                const accounts = await connection.getMultipleAccountsInfo(addresses, connection.commitment);

                const result: (fnk.AccountResult<{}> | null)[] = [];

                for (let i = 0; i < accounts.length; i++) {{
                    const address = addresses[i];
                    const account = accounts[i];

                    if (account) {{
                        if (fnk.equals(ID, account.owner)) {{
                            let buf = account.data;
                            let data = this.deserialize(buf);
                            result.push({{
                                address,
                                account,
                                data,
                            }});
                        }} else {{

                            result.push(null);
                        }}
                    }} else {{
                        result.push(null);
                    }}
                }}

                return result;
            }}
        }}",
        name_str,
        name_str,
        name_str,
        name_str,
    );

    let test_name = format_ident!("__ts_gen_test__account_ext_{}", name_str);
    let test_name_str = test_name.to_string();
    let result = quote! {
        #[cfg(feature = "ts-gen")]
        #[automatically_derived]
        #[allow(non_snake_case)]
        mod #test_name {
            use super::*;

            #[test]
            fn build() {
                 // Register action.
                crate::__ts_gen_test__setup::BUILD_CONTEXT.register_action(#test_name_str, file!(), move |action_context| {
                    action_context.add_account_type_extensions(#test_name_str, std::borrow::Cow::Borrowed(#type_extension)).unwrap();
                })
            }
        }
    };

    Ok(result)
}
