use crate::Result;
use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::{Expr, Lit, LitInt};

/// Unwraps an int literal from a generic literal.
pub fn unwrap_int_from_literal(lit: Lit) -> Result<LitInt> {
    match lit {
        Lit::Int(lit) => Ok(lit),
        v => Err(syn::Error::new(v.span(), "Expected integer literal")),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Unwraps an int literal from an expression.
pub fn unwrap_int_from_expr(expr: Expr) -> Result<LitInt> {
    match expr {
        Expr::Lit(lit) => match lit.lit {
            Lit::Int(lit) => Ok(lit),
            v => Err(syn::Error::new(v.span(), "Expected integer literal")),
        },
        _ => Err(syn::Error::new(expr.span(), "Expected integer literal")),
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Unwraps an ident from an expression.
pub fn unwrap_ident_from_expr(expr: Expr) -> Result<Ident> {
    match expr {
        Expr::Path(v) => {
            if v.path.segments.len() != 1 {
                return Err(syn::Error::new(v.span(), "Expected identifier"));
            }

            let ident = v.path.segments.first().unwrap().ident.clone();
            Ok(ident)
        }
        _ => Err(syn::Error::new(expr.span(), "Expected identifier")),
    }
}
