use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::parse_quote::ParseQuote;
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

pub struct CustomMetaList {
    pub list: Punctuated<CustomMeta, Token![,]>,
}

impl Parse for CustomMetaList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            list: <Punctuated<CustomMeta, Token![,]>>::parse(input)?,
        })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CustomMeta {
    pub name: Ident,
    pub eq_token: Option<Token![=]>,
    pub value: Option<Expr>,
}

impl Parse for CustomMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let eq_token = input.parse::<Option<Token![=]>>()?;
        let value = if eq_token.is_some() {
            Some(input.parse::<Expr>()?)
        } else {
            None
        };

        Ok(Self {
            name,
            eq_token,
            value,
        })
    }
}
