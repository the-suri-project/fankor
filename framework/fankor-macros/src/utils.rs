use crate::Result;
use syn::{Lit, LitInt, LitStr};

/// Unwraps an int literal from a generic literal.
pub fn unwrap_int_from_literal(lit: Lit) -> Result<LitInt> {
    match lit {
        Lit::Int(lit) => Ok(lit),
        v => Err(syn::Error::new(v.span(), "Expected integer literal")),
    }
}

/// Unwraps a string literal from a generic literal.
pub fn unwrap_string_from_literal(lit: Lit) -> Result<LitStr> {
    match lit {
        Lit::Str(lit) => Ok(lit),
        v => Err(syn::Error::new(v.span(), "Expected string literal")),
    }
}
