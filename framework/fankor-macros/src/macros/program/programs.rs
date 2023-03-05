use crate::fnk_syn::{FnkMetaArgumentList, FnkMetaType};
use crate::Result;
use convert_case::{Boundary, Case, Converter};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::{Attribute, Error, ItemEnum};

pub struct Program {
    pub name: Ident,
    pub snake_name: Ident,
    pub methods: Vec<ProgramMethod>,
    pub fallback_method_call: Option<TokenStream>,
    pub testable: bool,

    /// List of attributes to apply to the enum.
    pub attrs: Vec<Attribute>,
}

pub struct ProgramMethod {
    pub name: Ident,
    pub snake_name: Ident,
    pub return_type: Option<TokenStream>,
    pub boxed: bool,
    pub attrs: Vec<Attribute>,
}

impl Program {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the Field struct from the given attributes.
    pub fn from(mut args: FnkMetaArgumentList, item: ItemEnum) -> Result<Program> {
        args.error_on_duplicated()?;

        let fallback_method_call = args.pop_element("fallback", true)?.map(|v| match v.value {
            Some(v) => v.to_token_stream(),
            None => quote! { fallback(program_id, accounts, data) },
        });

        let testable = args.pop_plain("testable", true)?;

        args.error_on_unknown()?;

        if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
            return Err(Error::new(
                item.generics.span(),
                "program macro does not support generics in the enum declarations",
            ));
        }

        let name = item.ident.clone();
        let case_converter = Converter::new()
            .from_case(Case::Pascal)
            .to_case(Case::Snake)
            .remove_boundary(Boundary::LowerDigit)
            .remove_boundary(Boundary::UpperDigit);

        let snake_name = format_ident!(
            "{}",
            case_converter.convert(name.to_string()),
            span = name.span()
        );

        let mut program = Program {
            name,
            snake_name,
            methods: vec![],
            fallback_method_call,
            testable,
            attrs: Vec::new(),
        };

        program.parse_methods(item)?;

        Ok(program)
    }

    fn parse_methods(&mut self, item: ItemEnum) -> Result<()> {
        let case_converter = Converter::new()
            .from_case(Case::Pascal)
            .to_case(Case::Snake)
            .remove_boundary(Boundary::LowerDigit)
            .remove_boundary(Boundary::UpperDigit);

        for variant in item.variants {
            let method_name = variant.ident.clone();

            let mut return_type = None;
            let mut boxed = false;
            let mut attrs = Vec::new();

            for attribute in variant.attrs {
                let attribute_span = attribute.span();

                if attribute.path.is_ident("return_type") {
                    let result = if let Ok(value) = FnkMetaType::parse.parse2(attribute.tokens) {
                        value.ty.to_token_stream()
                    } else {
                        return Err(Error::new(
                            attribute_span,
                            "The correct pattern is #[return_type = <type>]",
                        ));
                    };

                    return_type = Some(result);
                } else if attribute.path.is_ident("boxed") {
                    if !attribute.tokens.is_empty() {
                        return Err(Error::new(
                            attribute_span,
                            "The correct pattern is #[boxed]",
                        ));
                    }

                    boxed = true;
                } else {
                    attrs.push(attribute);
                }
            }

            self.methods.push(ProgramMethod {
                snake_name: format_ident!(
                    "{}",
                    case_converter.convert(method_name.to_string()),
                    span = method_name.span()
                ),
                name: method_name,
                return_type,
                boxed,
                attrs,
            });
        }

        Ok(())
    }
}
