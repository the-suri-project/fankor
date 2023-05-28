use quote::quote;
use syn::{Error, Item};
use syn::spanned::Spanned;

use crate::fnk_syn::FnkMetaArgumentList;
use crate::Result;

pub fn processor(args: FnkMetaArgumentList, input: Item) -> Result<proc_macro::TokenStream> {
    // Process arguments.
    if !args.is_empty() {
        return Err(Error::new(
            input.span(),
            "fankor_base macro does not accept arguments",
        ));
    }

    let enum_discriminant_attr = if matches!(input, Item::Enum(_)) {
        quote! {
            #[derive(EnumDiscriminants)]
        }
    } else {
        quote! {}
    };

    let result = quote! {
        #enum_discriminant_attr
        #[derive(FankorSerialize, FankorDeserialize, FankorZeroCopy, TsGen)]
        #input
    };

    Ok(result.into())
}
