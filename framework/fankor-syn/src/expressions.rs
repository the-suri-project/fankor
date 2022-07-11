use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_str, Error, Expr, Lit, LitInt, LitStr};

use crate::Result;

/// Parses a string literal into an expression.
pub fn parse_expression_from_string(lit: LitStr) -> Result<TokenStream> {
    match parse_str::<Expr>(lit.value().as_str()) {
        Ok(v) => Ok(v.to_token_stream()),
        Err(e) => Err(Error::new(
            lit.span(),
            format!("Failed to parse expression: {}", e),
        )),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Unwraps a string literal from a generic literal.
pub fn unwrap_string_from_literal(lit: Lit) -> Result<LitStr> {
    match lit {
        Lit::Str(lit) => Ok(lit),
        v => Err(Error::new(v.span(), "Expected string literal")),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Unwraps an int literal from a generic literal.
pub fn unwrap_int_from_literal(lit: Lit) -> Result<LitInt> {
    match lit {
        Lit::Int(lit) => Ok(lit),
        v => Err(Error::new(v.span(), "Expected integer literal")),
    }
}
