use convert_case::{Boundary, Case, Converter};
use proc_macro2::Ident;
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::ops::RangeInclusive;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Error, Expr, ExprParen, ExprTuple, FnArg, GenericArgument,
    ImplItem, ItemImpl, Lit, MetaList, NestedMeta, PathArguments, RangeLimits, ReturnType, Type,
};

use crate::utils::unwrap_int_from_literal;
use crate::Result;

pub struct Program {
    pub name: Ident,
    pub snake_name: Ident,
    pub item: ItemImpl,
    pub methods: Vec<ProgramMethod>,
    pub fallback_method: Option<ProgramMethod>,

    /// Discriminants that cannot be used in the account list.
    pub removed_discriminants: Vec<RangeInclusive<u8>>,

    /// List of attributes to apply to the enum.
    pub attrs: Vec<Attribute>,
}

pub struct ProgramMethod {
    pub name: Ident,
    pub pascal_name: Ident,
    pub account_type: Type,
    pub argument_type: Option<Type>,
    pub result_type: Option<Type>,
    pub discriminant: u8,
    pub deprecated: bool,
    pub independent_validation: bool,
}

impl Program {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the Field struct from the given attributes.
    pub fn from(item: ItemImpl) -> Result<Program> {
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
            item: item.clone(),
            methods: vec![],
            fallback_method: None,
            removed_discriminants: Vec::new(),
            attrs: Vec::new(),
        };

        // Process attributes.
        for attr in &item.attrs {
            if attr.path.is_ident("removed_discriminants") {
                let elems = match parse_macro_input::parse::<ExprTuple>(attr.tokens.clone().into())
                {
                    Ok(v) => v.elems,
                    Err(e) => {
                        match parse_macro_input::parse::<ExprParen>(attr.tokens.clone().into()) {
                            Ok(v) => {
                                let mut res = Punctuated::new();
                                res.push(*v.expr);
                                res
                            }
                            Err(_) => return Err(Error::new(attr.span(), e.to_string())),
                        }
                    }
                };

                for el in elems {
                    match el {
                        Expr::Lit(v) => {
                            let value = unwrap_int_from_literal(v.lit)?.base10_parse()?;
                            program.removed_discriminants.push(value..=value);
                        }
                        Expr::Range(v) => {
                            if v.from.is_none() {
                                return Err(Error::new(v.span(), "Range must have a start value"));
                            }

                            if v.to.is_none() {
                                return Err(Error::new(v.span(), "Range must have an end value"));
                            }

                            let half_open = matches!(v.limits, RangeLimits::HalfOpen(_));

                            let span = v.span();
                            let from = match *v.from.unwrap() {
                                Expr::Lit(v) => unwrap_int_from_literal(v.lit)?.base10_parse()?,
                                _ => {
                                    return Err(Error::new(
                                        span,
                                        "Only literal values are allowed in ranges",
                                    ));
                                }
                            };

                            let to = match *v.to.unwrap() {
                                Expr::Lit(v) => unwrap_int_from_literal(v.lit)?.base10_parse()?,
                                _ => {
                                    return Err(Error::new(
                                        span,
                                        "Only literal values are allowed in ranges",
                                    ));
                                }
                            };

                            if half_open {
                                program.removed_discriminants.push(from..=to - 1);
                            } else {
                                program.removed_discriminants.push(from..=to);
                            }
                        }
                        _ => {
                            return Err(Error::new(el.span(), "Unknown argument"));
                        }
                    }
                }

                continue;
            }

            program.attrs.push(attr.clone());
        }

        // Validate removed_codes attribute.
        if !program.removed_discriminants.is_empty() {
            let mut prev = program.removed_discriminants.first().unwrap();

            assert!(
                prev.start() <= prev.end(),
                "Ranges must be defined in ascending order. {}..={} is not valid",
                prev.start(),
                prev.end()
            );

            for el in program.removed_discriminants.iter().skip(1) {
                assert!(
                    el.start() <= el.end(),
                    "Ranges must be defined in ascending order. {}..={} is not valid",
                    el.start(),
                    el.end()
                );
                assert!(
                    prev.end() < el.start(),
                    "Ranges cannot collide. {}..={} and {}..={} are colliding",
                    prev.start(),
                    prev.end(),
                    el.start(),
                    el.end()
                );
                assert_ne!(
                    prev.end() + 1,
                    *el.start(),
                    "Ranges can be collided. Replace {}..={} and {}..={} by {}..={}",
                    prev.start(),
                    prev.end(),
                    el.start(),
                    el.end(),
                    prev.start(),
                    el.end()
                );
                prev = el;
            }
        }

        program.parse_methods()?;

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

    fn parse_methods(&mut self) -> Result<()> {
        let case_converter = Converter::new()
            .from_case(Case::Snake)
            .to_case(Case::Pascal);

        let mut u8_index = 1u8;
        let mut last_deprecated = false;
        let mut used_discriminants = HashSet::new();

        for item in self.item.items.iter_mut() {
            let item = match item {
                ImplItem::Method(v) => v,
                _ => continue,
            };

            let method_name = item.sig.ident.clone();
            let mut is_fallback = false;

            if method_name == "fallback" {
                is_fallback = true;

                if self.fallback_method.is_some() {
                    return Err(Error::new(
                        item.span(),
                        "can only exist one fallback method",
                    ));
                }
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

                    let mut discriminant = None;
                    let mut deprecated = false;
                    let mut independent_validation = false;
                    let mut index = 0;

                    while index < item.attrs.len() {
                        let attribute = &item.attrs[index];

                        if attribute.path.is_ident("discriminant") {
                            let attribute = item.attrs.remove(index);
                            let attribute_span = attribute.span();

                            if discriminant.is_some() {
                                return Err(Error::new(
                                    attribute_span,
                                    "The discriminant attribute can only be used once",
                                ));
                            }

                            let path = &attribute.path;
                            let tokens = &attribute.tokens;
                            let tokens = quote! {#path #tokens};

                            let args = match parse_macro_input::parse::<MetaList>(tokens.into()) {
                                Ok(v) => v,
                                Err(_) => {
                                    return Err(Error::new(
                                        attribute_span,
                                        "The discriminant attribute expects one integer literal as arguments",
                                    ));
                                }
                            };

                            if args.nested.len() != 1 {
                                return Err(Error::new(
                                    attribute_span,
                                    "The discriminant attribute expects only one argument",
                                ));
                            }

                            // Check first argument is a literal string.
                            let first_argument = args.nested.first().unwrap();
                            match first_argument {
                                NestedMeta::Lit(v) => match v {
                                    Lit::Int(v) => {
                                        discriminant = Some(v.clone());
                                    }
                                    v => {
                                        return Err(Error::new(
                                            v.span(),
                                            "This must be a literal string",
                                        ));
                                    }
                                },
                                NestedMeta::Meta(v) => {
                                    return Err(Error::new(
                                        v.span(),
                                        "This must be a literal string",
                                    ));
                                }
                            }
                        } else if attribute.path.is_ident("deprecated") {
                            let attribute_span = attribute.span();

                            if deprecated {
                                return Err(Error::new(
                                    attribute_span,
                                    "The deprecated attribute can only be used once",
                                ));
                            }

                            deprecated = true;
                            index += 1;
                        } else if attribute.path.is_ident("independent_validation") {
                            let attribute = item.attrs.remove(index);
                            let attribute_span = attribute.span();

                            if independent_validation {
                                return Err(Error::new(
                                    attribute_span,
                                    "The independent_validation attribute can only be used once",
                                ));
                            }

                            independent_validation = true;
                        } else {
                            index += 1;
                        }
                    }

                    if !is_fallback {
                        if last_deprecated && discriminant.is_none() {
                            return Err(Error::new(
                                item.span(),
                                "The next method after a deprecated one must have the #[discriminant] attribute",
                            ));
                        }

                        last_deprecated = deprecated;
                    }

                    // Calculate the discriminant.
                    if let Some(v) = discriminant {
                        let new_value = v.base10_parse::<u8>()?;

                        u8_index = new_value;
                    }

                    if u8_index == 0 {
                        return Err(Error::new(
                            item.span(),
                            "Zero discriminant is reserved for uninitialized accounts, please provide another one",
                        ));
                    }

                    if used_discriminants.contains(&u8_index) {
                        return Err(Error::new(
                            item.span(),
                            format!("The discriminant attribute is already in use: {}", u8_index),
                        ));
                    }

                    used_discriminants.insert(u8_index);

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
                        discriminant: u8_index,
                        deprecated,
                        independent_validation,
                    });

                    u8_index += 1;
                }
                3 => {
                    let account_type = type_from_fn_arg(&arguments[1])?;
                    let argument_type = type_from_fn_arg(&arguments[2])?;
                    let result_type = type_from_fn_output(&item.sig.output)?;

                    let mut discriminant = None;
                    let mut deprecated = false;
                    let mut independent_validation = false;
                    let mut index = 0;
                    while index < item.attrs.len() {
                        let attribute = &item.attrs[index];

                        if attribute.path.is_ident("discriminant") {
                            let attribute = item.attrs.remove(index);
                            let attribute_span = attribute.span();

                            if discriminant.is_some() {
                                return Err(Error::new(
                                    attribute_span,
                                    "The discriminant attribute can only be used once",
                                ));
                            }

                            let path = &attribute.path;
                            let tokens = &attribute.tokens;
                            let tokens = quote! {#path #tokens};

                            let args = match parse_macro_input::parse::<MetaList>(tokens.into()) {
                                Ok(v) => v,
                                Err(_) => {
                                    return Err(Error::new(
                                        attribute_span,
                                        "The discriminant attribute expects one integer literal as arguments",
                                    ));
                                }
                            };

                            if args.nested.len() != 1 {
                                return Err(Error::new(
                                    attribute_span,
                                    "The discriminant attribute expects only one argument",
                                ));
                            }

                            // Check first argument is a literal string.
                            let first_argument = args.nested.first().unwrap();
                            match first_argument {
                                NestedMeta::Lit(v) => match v {
                                    Lit::Int(v) => {
                                        discriminant = Some(v.clone());
                                    }
                                    v => {
                                        return Err(Error::new(
                                            v.span(),
                                            "This must be a literal string",
                                        ));
                                    }
                                },
                                NestedMeta::Meta(v) => {
                                    return Err(Error::new(
                                        v.span(),
                                        "This must be a literal string",
                                    ));
                                }
                            }
                        } else if attribute.path.is_ident("deprecated") {
                            let attribute_span = attribute.span();

                            if deprecated {
                                return Err(Error::new(
                                    attribute_span,
                                    "The deprecated attribute can only be used once",
                                ));
                            }

                            deprecated = true;
                            index += 1;
                        } else if attribute.path.is_ident("independent_validation") {
                            let attribute = item.attrs.remove(index);
                            let attribute_span = attribute.span();

                            if independent_validation {
                                return Err(Error::new(
                                    attribute_span,
                                    "The independent_validation attribute can only be used once",
                                ));
                            }

                            independent_validation = true;
                        } else {
                            index += 1;
                        }
                    }

                    if !is_fallback {
                        if last_deprecated && discriminant.is_none() {
                            return Err(Error::new(
                                item.span(),
                                "The next method after a deprecated one must have the #[discriminant] attribute",
                            ));
                        }

                        last_deprecated = deprecated;
                    }

                    // Calculate the discriminant.
                    if let Some(v) = &discriminant {
                        let new_value = v.base10_parse::<u8>()?;

                        u8_index = new_value;
                    }

                    if u8_index == 0 {
                        return Err(Error::new(
                            item.span(),
                            "Zero discriminant is reserved for uninitialized accounts, please provide another one".to_string(),
                        ));
                    }

                    if used_discriminants.contains(&u8_index) {
                        return Err(Error::new(
                            item.span(),
                            format!("The discriminant attribute is already in use: {}", u8_index),
                        ));
                    }

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
                        discriminant: u8_index,
                        deprecated,
                        independent_validation,
                    };

                    if is_fallback {
                        self.fallback_method = Some(method);

                        if discriminant.is_some() {
                            return Err(Error::new(
                                item.span(),
                                "The fallback method cannot have the #[discriminant] attribute",
                            ));
                        }
                    } else {
                        used_discriminants.insert(u8_index);

                        self.methods.push(method);

                        u8_index += 1;
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
                                "FankorResult's generic must be a type",
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
