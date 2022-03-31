use quote::quote;
use syn::parse_macro_input;
use syn::AttributeArgs;

#[proc_macro]
pub fn setup(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);

    assert!(args.is_empty(), "setup macro takes no arguments");

    let result = quote! {
        #[cfg(all(test, feature = "builder-tests"))]
        pub mod __internal__idl_builder_tests__root {
            // Helper to execute the tests in sequence.
            ::fankor::build::prelude::lazy_static! {
                pub static ref IDL_CONTEXT: ::std::sync::Arc<::fankor::build::IdlContext> = ::std::sync::Arc::new(::fankor::build::IdlContext::new());
            }

            // ----------------------------------------------------------------
            // ----------------------------------------------------------------
            // ----------------------------------------------------------------

            /// This is the main build function.
            #[test]
            fn build() {
                IDL_CONTEXT.build();
            }
        }
    };

    result.into()
}
