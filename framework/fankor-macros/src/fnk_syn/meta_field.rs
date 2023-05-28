use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

pub struct FnkMetaField {
    pub name: Ident,
    pub sub_name: Option<(Token![::], Ident)>,
    pub eq_token: Option<Token![=]>,
    pub value: Option<Expr>,
}

impl Parse for FnkMetaField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;

        let sub_name = if let Ok(token_colon2) = input.parse::<Token![::]>() {
            let sub_name = input.parse::<Ident>()?;
            Some((token_colon2, sub_name))
        } else {
            None
        };

        let eq_token = input.parse::<Option<Token![=]>>()?;
        let value = if eq_token.is_some() {
            Some(input.parse::<Expr>()?)
        } else {
            None
        };

        Ok(Self {
            name,
            sub_name,
            eq_token,
            value,
        })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct FnkMetaFieldWithError {
    pub meta: FnkMetaField,
    pub at_token: Option<Token![@]>,
    pub error: Option<Expr>,
}

impl Parse for FnkMetaFieldWithError {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta = input.parse::<FnkMetaField>()?;
        let at_token = input.parse::<Option<Token![@]>>()?;
        let error = if at_token.is_some() {
            Some(input.parse::<Expr>()?)
        } else {
            None
        };

        Ok(Self {
            meta,
            at_token,
            error,
        })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct FnkMetaFieldListWithErrors {
    pub list: Punctuated<FnkMetaFieldWithError, Token![,]>,
}

impl Parse for FnkMetaFieldListWithErrors {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            list: <Punctuated<FnkMetaFieldWithError, Token![,]>>::parse_terminated(input)?,
        })
    }
}
