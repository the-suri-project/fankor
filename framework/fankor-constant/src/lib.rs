use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, Error, Item};

use fankor_syn::Result;

use crate::attributes::ConstantAttributes;

mod attributes;

#[proc_macro_attribute]
pub fn constant(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);

    match processor(args, input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

fn processor(args: AttributeArgs, input: Item) -> Result<proc_macro::TokenStream> {
    // Process arguments.
    let args = ConstantAttributes::from(args)?;

    // Process input.
    let (constant_name, constant_type) = match &input {
        Item::Const(v) => (&v.ident, &v.ty),
        Item::Static(v) => (&v.ident, &v.ty),
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
        .unwrap_or(constant_name.to_string());
    let module_name = format_ident!(
        "__internal__idl_builder_tests__constants__{}",
        constant_name
    );
    let module_name_str = module_name.to_string();

    let result = quote! {
        #input

        #[cfg(all(test, feature = "builder-tests"))]
        #[allow(non_snake_case)]
        mod #module_name {
            #[test]
            fn build() {
                // Register action.
                crate::__internal__idl_builder_tests__root::IDL_CONTEXT.register_action(#module_name_str, file!(), move |idl_build_context| {
                    let name = #constant_name_str;
                    let value = <#constant_type as ::fankor::build::types::IdlTypeMappable>::map_to_idl(&super::#constant_name);
                    let ty = <#constant_type as ::fankor::build::types::IdlTypeMappable>::idl_type();

                    // Add the constant.
                    idl_build_context.add_constant(name.to_string(), ty, value).unwrap_or_else(|_| ::std::panic!("Duplicated constant name '{}'. The IDL requires all constants have a unique name. Use the alias attribute to change one of them only for the IDL: #[constant(alias = \"idl_name\")]", name));
                })
            }
        }
    };

    Ok(result.into())
}
