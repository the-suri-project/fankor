use crate::Result;
use quote::quote;
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, Item};

pub fn processor(args: AttributeArgs, input: Item) -> Result<proc_macro::TokenStream> {
    // Process arguments.
    if !args.is_empty() {
        return Err(Error::new(
            input.span(),
            "base macro does not accept arguments",
        ));
    }

    let enum_discriminant_attr = if matches!(input, Item::Enum(_)) {
        quote! {
            #[derive(EnumDiscriminants)]
        }
    } else {
        quote! {}
    };

    // TODO add TsGen
    let result = quote! {
        #enum_discriminant_attr
        #[derive(FankorSerialize, FankorDeserialize, FankorZeroCopy)]
        #input
    };

    Ok(result.into())
}
