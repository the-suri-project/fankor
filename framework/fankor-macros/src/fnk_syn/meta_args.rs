use crate::utils::{unwrap_ident_from_expr, unwrap_int_from_expr};
use crate::Result;
use proc_macro2::{Ident, Span};
use std::fmt::Display;
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::parse_quote::ParseQuote;
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

pub struct FnkMetaArgument {
    pub name: Ident,
    pub value: Option<Expr>,
}

impl Parse for FnkMetaArgument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let eq_token = input.parse::<Option<Token![=]>>()?;
        let value = if eq_token.is_some() {
            Some(input.parse::<Expr>()?)
        } else {
            None
        };

        Ok(Self { name, value })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct FnkMetaArgumentList {
    pub list_span: Span,
    pub list: Vec<FnkMetaArgument>,
}

impl FnkMetaArgumentList {
    // GETTERS ----------------------------------------------------------------

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    // METHODS ----------------------------------------------------------------

    pub fn pop(
        &mut self,
        name: &str,
        with_value: Option<bool>,
        optional: bool,
    ) -> Result<Option<FnkMetaArgument>> {
        let position = match self.list.iter().position(|v| v.name == name) {
            Some(v) => v,
            None => {
                return if optional {
                    Ok(None)
                } else {
                    Err(syn::Error::new(
                        self.list_span,
                        format!("Attribute {} is required", name),
                    ))
                }
            }
        };

        let element = self.list.remove(position);

        if let Some(with_value) = with_value {
            if with_value {
                if element.value.is_none() {
                    return Err(syn::Error::new(
                        element.name.span(),
                        format!("Attribute {} must have value", element.name),
                    ));
                }
            } else if element.value.is_some() {
                return Err(syn::Error::new(
                    element.name.span(),
                    format!("Attribute {} does not accept a value", element.name),
                ));
            }
        }

        Ok(Some(element))
    }

    pub fn pop_plain(&mut self, name: &str, optional: bool) -> Result<bool> {
        Ok(self.pop(name, Some(false), optional)?.is_some())
    }

    pub fn pop_element(&mut self, name: &str, optional: bool) -> Result<Option<FnkMetaArgument>> {
        self.pop(name, None, optional)
    }

    pub fn pop_number<N>(&mut self, name: &str, optional: bool) -> Result<Option<N>>
    where
        N: FromStr,
        N::Err: Display,
    {
        let element = match self.pop(name, Some(true), optional)? {
            Some(v) => v,
            None => return Ok(None),
        };

        let value = match element.value {
            Some(v) => unwrap_int_from_expr(v)?,
            None => {
                return Err(syn::Error::new(
                    element.name.span(),
                    format!("Attribute {} requires a u16 value", element.name),
                ))
            }
        };

        Ok(Some(value.base10_parse().map_err(|e| {
            syn::Error::new(
                element.name.span(),
                format!("Invalid value for {}: {}", element.name, e),
            )
        })?))
    }

    pub fn pop_ident(&mut self, name: &str, optional: bool) -> Result<Option<Ident>> {
        let element = match self.pop(name, Some(true), optional)? {
            Some(v) => v,
            None => return Ok(None),
        };

        let value = match element.value {
            Some(v) => unwrap_ident_from_expr(v)?,
            None => {
                return Err(syn::Error::new(
                    element.name.span(),
                    format!("Attribute {} requires an ident value", element.name),
                ))
            }
        };

        Ok(Some(value))
    }

    pub fn error_on_duplicated(&self) -> Result<()> {
        for (i, arg1) in self.list.iter().enumerate() {
            for arg2 in self.list.iter().skip(i + 1) {
                if arg1.name == arg2.name {
                    return Err(syn::Error::new(
                        arg1.name.span(),
                        format!("Duplicated attribute: {}", arg1.name),
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn error_on_unknown(&self) -> Result<()> {
        if let Some(arg) = self.list.last() {
            return Err(syn::Error::new(
                arg.name.span(),
                format!("Unknown attribute: {}", arg.name),
            ));
        }

        Ok(())
    }
}

impl Parse for FnkMetaArgumentList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let list_span = input.span();
        let list = <Punctuated<FnkMetaArgument, Token![,]>>::parse(input)?;

        Ok(Self {
            list_span,
            list: list.into_iter().collect(),
        })
    }
}
