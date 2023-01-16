use crate::Result;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Item};

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    let (constant_name, constant_value) = match &input {
        Item::Const(v) => (&v.ident, &v.expr),
        _ => {
            return Err(Error::new(
                input.span(),
                "This attribute can only be used on a const.",
            ))
        }
    };

    let constant_name_str = constant_name.to_string();
    let constant_test_name = format_ident!("__ts_gen_test__constant_{}", constant_name);
    let constant_test_name_str = constant_test_name.to_string();
    let result = quote! {
        #input

        #[cfg(all(test, feature = "ts-gen"))]
        #[automatically_derived]
        #[allow(non_snake_case)]
        pub mod #constant_test_name {
            #[test]
            fn build() {
                 // Register action.
                crate::__ts_gen_test__setup::BUILD_CONTEXT.register_action(#constant_test_name_str, file!(), move |action_context| {
                    action_context.add_constant(#constant_name_str, #constant_value).unwrap();
                })
            }
        }
    };

    Ok(result.into())
}
