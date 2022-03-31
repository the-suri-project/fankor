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
                static ref IDL_BUILD_CONTEXT_MUTEX: ::std::sync::Mutex<::fankor::build::IdlBuildContext> = ::std::sync::Mutex::new(::fankor::build::IdlBuildContext::new());
            }

            pub fn lock_idl_build_context<F>(test_name: &str, f: F)
            where F: FnOnce(&mut ::fankor::build::IdlBuildContext) -> () + ::std::panic::UnwindSafe {
                let mut idl_build_context = IDL_BUILD_CONTEXT_MUTEX.lock().unwrap();

                // Checks whether another error has already been reported and panics if so.
                if let Some(error_file) = idl_build_context.error_file() {
                    ::std::panic!("Another builder test failed: {}", error_file);
                }

                // Executes the function and captures any error to report it to others.
                let arg: &mut ::fankor::build::IdlBuildContext = &mut idl_build_context;
                let mut arg = ::std::panic::AssertUnwindSafe(arg);
                let result = ::std::panic::catch_unwind(move || {
                    f(*arg)
                });

                if let Err(err) = result {
                    idl_build_context.set_error_file(format!("{}({})", file!(), test_name));
                    ::std::panic::resume_unwind(err);
                }
            }
        }
    };

    result.into()
}
