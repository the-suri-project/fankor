use convert_case::{Boundary, Case, Converter};
use fankor_syn::fankor::FankorInstructionsConfig;
use proc_macro2::Ident;
use quote::format_ident;
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{Error, FnArg, GenericArgument, ImplItem, ItemImpl, PathArguments, ReturnType, Type};

use crate::utils::generate_discriminator;
use crate::Result;

pub struct Program {
    pub name: Ident,
    pub snake_name: Ident,
    pub item: ItemImpl,
    pub methods: Vec<ProgramMethod>,
    pub fallback_method: Option<ProgramMethod>,
}

pub struct ProgramMethod {
    pub name: Ident,
    pub pascal_name: Ident,
    pub account_type: Type,
    pub argument_type: Option<Type>,
    pub result_type: Option<Type>,
    pub discriminator: Vec<u8>,
}

impl Program {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the Field struct from the given attributes.
    pub fn from(item: ItemImpl, config: &FankorInstructionsConfig) -> Result<Program> {
        let name = Self::verify_impl_and_get_name(&item)?;
        let case_converter = Converter::new()
            .from_case(Case::Pascal)
            .to_case(Case::Snake)
            .remove_boundary(Boundary::LowerDigit)
            .remove_boundary(Boundary::UpperDigit);

        let snake_name = format_ident!(
            "{}",
            case_converter.convert(&name.to_string()),
            span = name.span()
        );

        let mut program = Program {
            name,
            snake_name,
            item,
            methods: vec![],
            fallback_method: None,
        };

        program.parse_methods(config)?;

        // Check discriminators are unique.
        let mut discriminators = HashSet::new();
        for method in &program.methods {
            if !discriminators.insert(format!("{:?}", method.discriminator)) {
                return Err(Error::new(
                    method.name.span(),
                    format!("The discriminator for {} is not unique. Please increase discriminator size or change its name", method.name),
                ));
            }
        }

        Ok(program)
    }

    fn verify_impl_and_get_name(item: &ItemImpl) -> Result<Ident> {
        if let Some((_, v, _)) = &item.trait_ {
            return Err(Error::new(
                v.span(),
                "program macro does not support traits in the impl declarations",
            ));
        }

        if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
            return Err(Error::new(
                item.generics.span(),
                "program macro does not support generics in the impl declarations",
            ));
        }

        match &(*item.self_ty) {
            Type::Path(v) => {
                if let Some(v) = &v.qself {
                    return Err(Error::new(
                        v.ty.span(),
                        "program macro does not support this section",
                    ));
                }
                if v.path.segments.len() != 1 {
                    return Err(Error::new(
                        v.path.span(),
                        "program macro only supports ident paths",
                    ));
                }

                let segment = &v.path.segments[0];
                if !segment.arguments.is_empty() {
                    return Err(Error::new(
                        v.path.span(),
                        "program macro does not support this section",
                    ));
                }

                Ok(segment.ident.clone())
            }
            _ => Err(Error::new(
                item.self_ty.span(),
                "program macro only supports a name as the type in the impl declarations",
            )),
        }
    }

    fn parse_methods(&mut self, config: &FankorInstructionsConfig) -> Result<()> {
        let case_converter = Converter::new()
            .from_case(Case::Snake)
            .to_case(Case::Pascal);

        let mut fallbacks = 0;
        for item in self.item.items.iter_mut() {
            let item = match item {
                ImplItem::Method(v) => v,
                _ => continue,
            };

            let method_name = item.sig.ident.clone();
            let method_name_str = method_name.to_string();
            let mut is_fallback = false;

            if method_name == "fallback" {
                fallbacks += 1;
                is_fallback = true;
            }

            if fallbacks > 1 {
                return Err(Error::new(
                    item.span(),
                    "the fallback attribute can only be used once",
                ));
            }

            let arguments = &item.sig.inputs;
            match arguments.len() {
                2 => {
                    if is_fallback {
                        return Err(Error::new(
                            item.span(),
                            "incorrect signature for the fallback method",
                        ));
                    }

                    let account_type = type_from_fn_arg(&arguments[1])?;
                    let result_type = type_from_fn_output(&item.sig.output)?;
                    let discriminator =
                        if let Some(discriminator) = config.get_discriminator(&method_name_str) {
                            discriminator
                        } else {
                            let discriminator = generate_discriminator(&method_name_str);
                            discriminator[..config.discriminator_size.unwrap() as usize].to_vec()
                        };

                    self.methods.push(ProgramMethod {
                        pascal_name: format_ident!(
                            "{}",
                            case_converter.convert(&method_name.to_string()),
                            span = method_name.span()
                        ),
                        name: method_name,
                        account_type,
                        argument_type: None,
                        result_type,
                        discriminator,
                    });
                }
                3 => {
                    let account_type = type_from_fn_arg(&arguments[1])?;
                    let argument_type = type_from_fn_arg(&arguments[2])?;
                    let result_type = type_from_fn_output(&item.sig.output)?;
                    let discriminator =
                        if let Some(discriminator) = config.get_discriminator(&method_name_str) {
                            discriminator
                        } else {
                            let discriminator = generate_discriminator(&method_name_str);
                            discriminator[..config.discriminator_size.unwrap() as usize].to_vec()
                        };

                    let method = ProgramMethod {
                        pascal_name: format_ident!(
                            "{}",
                            case_converter.convert(&method_name.to_string()),
                            span = method_name.span()
                        ),
                        name: method_name,
                        account_type,
                        argument_type: Some(argument_type),
                        result_type,
                        discriminator,
                    };

                    if is_fallback {
                        self.fallback_method = Some(method);
                    } else {
                        self.methods.push(method);
                    }
                }
                _ => {
                    return Err(Error::new(
                        item.span(),
                        "program macro does not accept this signature",
                    ))
                }
            }
        }

        Ok(())
    }
}

fn type_from_fn_arg(arg: &FnArg) -> Result<Type> {
    match arg {
        FnArg::Typed(v) => Ok((*v.ty).clone()),
        FnArg::Receiver(v) => Err(Error::new(v.span(), "incorrect argument type")),
    }
}

fn type_from_fn_output(arg: &ReturnType) -> Result<Option<Type>> {
    match arg {
        ReturnType::Default => Err(Error::new(
            arg.span(),
            "instructions must return a FankorResult",
        )),
        ReturnType::Type(_, ty) => match &**ty {
            Type::Path(v) => {
                let last_arg = v.path.segments.last().unwrap();
                if &last_arg.ident.to_string() != "FankorResult" {
                    return Err(Error::new(
                        v.span(),
                        "instructions must always return a FankorResult",
                    ));
                }

                match &last_arg.arguments {
                    PathArguments::AngleBracketed(v) => {
                        if v.args.len() != 1 {
                            return Err(Error::new(
                                last_arg.span(),
                                "FankorResult has only one generic",
                            ));
                        }

                        let first = v.args.first().unwrap();
                        match first {
                            GenericArgument::Type(v) => match v {
                                Type::Tuple(v1) => {
                                    if v1.elems.is_empty() {
                                        Ok(None)
                                    } else {
                                        Ok(Some(v.clone()))
                                    }
                                }
                                _ => Ok(Some(v.clone())),
                            },
                            _ => Err(Error::new(
                                last_arg.span(),
                                "The FankorResult's generic must be a type",
                            )),
                        }
                    }
                    _ => Err(Error::new(
                        last_arg.span(),
                        "FankorResult must have a generic",
                    )),
                }
            }
            _ => Err(Error::new(
                ty.span(),
                "instructions must always return a FankorResult",
            )),
        },
    }
}
