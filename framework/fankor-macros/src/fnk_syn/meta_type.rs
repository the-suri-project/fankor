use syn::{Token, Type};
use syn::parse::{Parse, ParseStream};

pub struct FnkMetaType {
    pub eq_token: Token![=],
    pub ty: Type,
}

impl Parse for FnkMetaType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let eq_token = input.parse::<Token![=]>()?;
        let ty = input.parse::<Type>()?;

        Ok(Self { eq_token, ty })
    }
}
