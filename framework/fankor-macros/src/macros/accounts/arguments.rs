use std::ops::RangeInclusive;
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, spanned::Spanned, AttributeArgs, Error, Expr, ExprParen, ExprTuple,
    ItemEnum, RangeLimits,
};

use crate::utils::unwrap_int_from_literal;
use crate::Result;

pub struct AccountsArguments {
    /// Discriminants that cannot be used in the account list.
    pub removed_discriminants: Vec<RangeInclusive<u8>>,

    /// List of attributes to apply to the enum.
    pub attrs: Vec<syn::Attribute>,
}

impl AccountsArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the AccountsArguments struct from the given attributes.
    pub fn from(args: AttributeArgs, enum_item: &ItemEnum) -> Result<AccountsArguments> {
        let mut result = AccountsArguments {
            removed_discriminants: Vec::new(),
            attrs: Vec::new(),
        };

        assert!(args.is_empty(), "setup macro takes no arguments");

        // Process attributes.
        for attr in &enum_item.attrs {
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
                            result.removed_discriminants.push(value..=value);
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
                                result.removed_discriminants.push(from..=to - 1);
                            } else {
                                result.removed_discriminants.push(from..=to);
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
        if !result.removed_discriminants.is_empty() {
            let mut prev = result.removed_discriminants.first().unwrap();

            assert!(
                prev.start() <= prev.end(),
                "Ranges must be defined in ascending order. {}..={} is not valid",
                prev.start(),
                prev.end()
            );

            for el in result.removed_discriminants.iter().skip(1) {
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

    /// Checks whether the given discriminant is present in the removed_discriminants list or not.
    pub fn contains_removed_discriminant(&self, discriminant: u8) -> bool {
        for range in &self.removed_discriminants {
            if range.contains(&discriminant) {
                return true;
            }
        }

        false
    }
}
