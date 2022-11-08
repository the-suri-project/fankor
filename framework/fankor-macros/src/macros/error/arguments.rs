use quote::ToTokens;
use std::ops::RangeInclusive;
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, spanned::Spanned, AttributeArgs, Error, Expr, ExprParen, ExprTuple,
    ItemEnum, LitInt, Meta, NestedMeta, RangeLimits,
};

use fankor_syn::expressions::unwrap_int_from_literal;

use crate::Result;

pub struct ErrorArguments {
    /// The starting offset of the error list.
    pub offset: Option<LitInt>,

    /// Codes that cannot be used in the error list.
    pub removed_codes: Vec<RangeInclusive<u32>>,

    /// List of attributes to apply to the enum.
    pub attrs: Vec<syn::Attribute>,
}

impl ErrorArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ErrorArguments struct from the given attributes.
    pub fn from(args: AttributeArgs, enum_item: &ItemEnum) -> Result<ErrorArguments> {
        let mut result = ErrorArguments {
            offset: None,
            removed_codes: Vec::new(),
            attrs: Vec::new(),
        };

        for arg in args {
            match arg {
                NestedMeta::Meta(v) => match v {
                    Meta::NameValue(meta) => {
                        if meta.path.is_ident("offset") {
                            result.offset = Some(unwrap_int_from_literal(meta.lit)?);
                        } else {
                            return Err(Error::new(
                                meta.path.span(),
                                format!("Unknown attribute: {}", meta.path.to_token_stream()),
                            ));
                        }
                    }
                    Meta::List(v) => return Err(Error::new(v.span(), "Unknown argument")),
                    Meta::Path(v) => return Err(Error::new(v.span(), "Unknown argument")),
                },
                NestedMeta::Lit(v) => {
                    return Err(Error::new(v.span(), "Unknown argument"));
                }
            }
        }

        // Process attributes.
        for attr in &enum_item.attrs {
            if attr.path.is_ident("removed_codes") {
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
                            result.removed_codes.push(value..=value);
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
                                result.removed_codes.push(from..=to - 1);
                            } else {
                                result.removed_codes.push(from..=to);
                            }
                        }
                        _ => {
                            return Err(Error::new(el.span(), "Unknown argument"));
                        }
                    }
                }

                continue;
            }

            result.attrs.push(attr.clone());
        }

        // Validate removed_codes attribute.
        if !result.removed_codes.is_empty() {
            let mut prev = result.removed_codes.first().unwrap();

            assert!(
                prev.start() <= prev.end(),
                "Ranges must be defined in ascending order. {}..={} is not valid",
                prev.start(),
                prev.end()
            );

            for el in result.removed_codes.iter().skip(1) {
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

        Ok(result)
    }

    // METHODS ----------------------------------------------------------------

    /// Checks whether the given code is present in the removed_codes list or not.
    pub fn contains_removed_code(&self, code: u32) -> bool {
        for range in &self.removed_codes {
            if range.contains(&code) {
                return true;
            }
        }

        false
    }
}
