use quote::{format_ident, quote};
use syn::{Error, Item};
use syn::spanned::Spanned;

use crate::fnk_syn::FnkMetaArgumentList;
use crate::Result;

pub fn processor(args: FnkMetaArgumentList, input: Item) -> Result<proc_macro::TokenStream> {
    // Process arguments.
    if !args.is_empty() {
        return Err(Error::new(
            input.span(),
            "constant macro does not accept arguments",
        ));
    }

    let constant_name = match &input {
        Item::Const(v) => &v.ident,
        _ => {
            return Err(Error::new(
                input.span(),
                "This attribute can only be used on a const.",
            ));
        }
    };

    let constant_name_str = constant_name.to_string();
    let constant_test_name = format_ident!("__ts_gen_test__constant_{}", constant_name);
    let constant_test_name_str = constant_test_name.to_string();
    let result = quote! {
        #input

        #[cfg(feature = "ts-gen")]
        #[automatically_derived]
        #[allow(non_snake_case)]
        mod #constant_test_name {
            use super::*;

            #[test]
            fn build() {
                 // Register action.
                crate::__ts_gen_test__setup::BUILD_CONTEXT.register_action(#constant_test_name_str, file!(), move |action_context| {
                    action_context.add_constant(#constant_name_str, #constant_name).unwrap();
                })
            }
        }
    };

    Ok(result.into())
}
