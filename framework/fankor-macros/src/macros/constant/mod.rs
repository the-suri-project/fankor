mod attributes;

use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, Item};

use crate::macros::constant::attributes::ConstantAttributes;
use crate::utils::{string_to_hash, write_task_hash_to_file};
use fankor_syn::Result;

pub fn processor(args: AttributeArgs, input: Item) -> Result<proc_macro::TokenStream> {
    // Process arguments.
    let args = ConstantAttributes::from(args)?;

    // Process input.
    let constant_name = match &input {
        Item::Const(v) => &v.ident,
        Item::Static(v) => &v.ident,
        _ => {
            return Err(Error::new(
                input.span(),
                "constant macro can only be applied to const or static declarations",
            ));
        }
    };

    // Build output.
    let constant_name_str = args
        .alias
        .map(|v| v.value())
        .unwrap_or_else(|| constant_name.to_string());
    let module_name = format_ident!("__internal__idl_builder_test__constant__{}", constant_name);
    let module_name_str = module_name.to_string();
    let task_hash = string_to_hash(&module_name_str);

    let result = quote! {
        #input

        #[cfg(all(test, feature = "builder"))]
        #[allow(non_snake_case)]
        mod #module_name {
            #[test]
            fn build() {
                // Register action.
                crate::__internal__idl_builder_test__root::IDL_CONTEXT.register_action(#task_hash, #module_name_str, file!(), move |idl_build_context| {
                    let name = #constant_name_str;

                    // Add the constant.
                    idl_build_context.add_constant(name.to_string(), &super::#constant_name).unwrap_or_else(|_| ::std::panic!("Duplicated constant name '{}'. The IDL requires all constants have a unique name. Use the alias attribute to change one of them only for the IDL: #[constant(alias = \"idl_name\")]", name));
                })
            }
        }
    };

    write_task_hash_to_file(&task_hash);

    Ok(result.into())
}
