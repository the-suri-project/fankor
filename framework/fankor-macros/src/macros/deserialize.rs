use borsh_derive_internal::{enum_de, struct_de};
use fankor_syn::Result;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, ItemEnum, ItemImpl, ItemStruct, ItemUnion};

pub fn processor(input: proc_macro::TokenStream) -> Result<proc_macro::TokenStream> {
    let cratename = Ident::new("borsh", Span::call_site());

    let (res, initial_where_clause) = if let Ok(input) = syn::parse::<ItemStruct>(input.clone()) {
        let initial_where_clause = input.generics.where_clause.clone();

        (struct_de(&input, cratename)?, initial_where_clause)
    } else if let Ok(input) = syn::parse::<ItemEnum>(input.clone()) {
        let initial_where_clause = input.generics.where_clause.clone();

        (enum_de(&input, cratename)?, initial_where_clause)
    } else if syn::parse::<ItemUnion>(input).is_ok() {
        unimplemented!()
    } else {
        // Derive macros can only be defined on structs, enums, and unions.
        unreachable!()
    };

    let mut impl_block = syn::parse2::<ItemImpl>(res).unwrap();
    impl_block.generics.where_clause = initial_where_clause;

    Ok(proc_macro::TokenStream::from(quote! {
        #impl_block
    }))
}
