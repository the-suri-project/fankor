use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, Item};

// TODO add alias attribute.
// TODO add get attribute.
#[proc_macro_attribute]
pub fn constant(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);

    assert!(args.is_empty(), "constant macro takes no arguments");

    let constant_name = match &input {
        Item::Const(v) => &v.ident,
        Item::Static(v) => &v.ident,
        _ => panic!("constant macro can only be applied to const or static declarations"),
    };

    let constant_name_str = constant_name.to_string();
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
            fn build_js() {
                let name = #constant_name_str;
                let value = super::#constant_name;

                // Lock test.
                crate::__internal__idl_builder_tests__root::lock_idl_build_context(#module_name_str, |idl_build_context| {
                    // Serialize value.
                    let serialized_value = ::fankor::build::prelude::serde_json::to_value(value).unwrap_or_else(|e| ::std::panic!("Cannot serialize constant {}'s value: {}", name, e));

                    idl_build_context.add_constant(name.to_string(), serialized_value).unwrap_or_else(|_| ::std::panic!("Duplicated constant name '{}'. The IDL requires all constants have a unique name. Use the alias attribute to change one of them only for the IDL: #[constant(alias = \"idl_name\")]", name));
                });
            }
        }
    };

    result.into()
}
