use convert_case::{Boundary, Case, Converter};
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{parse_quote, AttributeArgs, Error, Item};

use crate::Result;

use crate::macros::accounts::arguments::AccountsArguments;
use crate::macros::accounts::variant::AccountVariant;
use crate::macros::enum_discriminants::get_discriminant;

mod arguments;
mod variant;

pub fn processor(args: AttributeArgs, input: Item) -> Result<proc_macro::TokenStream> {
    // Process input.
    let enum_item = match input {
        Item::Enum(v) => v,
        _ => {
            return Err(Error::new(
                input.span(),
                "error_code macro can only be applied to enum declarations",
            ));
        }
    };

    // Process arguments.
    let arguments = AccountsArguments::from(args, &enum_item)?;

    let name = enum_item.ident;
    let name_str = name.to_string();
    let discriminant_name = format_ident!("{}Discriminant", name);
    let attrs = &arguments.attrs;

    assert!(
        !enum_item.variants.is_empty(),
        "Accounts enum must have at least one variant"
    );

    // Ensure first variant has discriminant >= 1.
    let discriminant = get_discriminant::<u8>(&enum_item.variants[0])?;

    // Parse fields taking into account whether any variant is deprecated or not.
    let mut variants = enum_item
        .variants
        .into_iter()
        .map(AccountVariant::from)
        .collect::<Result<Vec<_>>>()?;

    match discriminant {
        Some(v) => {
            if v == 0 {
                return Err(Error::new(
                    variants[0].name.span(),
                    "The zero discriminant is reserved",
                ));
            }
        }
        None => {
            // Add initial discriminant.
            if arguments.accounts_type_name.is_none() {
                variants[0]
                    .attributes
                    .push(parse_quote!(#[discriminant = 1]));
            }
        }
    }

    let visibility = enum_item.vis;

    let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();

    // Generate code.
    let mut used_discriminants = HashSet::new();
    let mut final_enum_variants = Vec::with_capacity(variants.len());
    let mut unwrap_methods = Vec::with_capacity(variants.len());
    let mut as_ref_methods = Vec::with_capacity(variants.len());
    let mut as_mut_methods = Vec::with_capacity(variants.len());
    let mut from_methods = Vec::with_capacity(variants.len());
    let mut serialize_entries = Vec::with_capacity(variants.len());
    let mut deserialize_entries = Vec::with_capacity(variants.len());
    let mut discriminants_as_list = Vec::with_capacity(variants.len());
    let mut variant_consts = Vec::with_capacity(variants.len());
    for variant in &variants {
        let AccountVariant {
            name: variant_name,
            attributes,
            ..
        } = &variant;

        let const_name = format_ident!("{}Discriminant", variant_name);

        let case_converter = Converter::new()
            .from_case(Case::Pascal)
            .to_case(Case::Snake)
            .remove_boundary(Boundary::LowerDigit)
            .remove_boundary(Boundary::UpperDigit);

        final_enum_variants.push(quote! {
            #(#attributes)*
            #variant_name(#variant_name)
        });

        let method_name = case_converter.convert(variant_name.to_string());

        unwrap_methods.push({
            let method_name = format_ident!("unwrap_{}", method_name, span = variant_name.span());

            quote! {
                pub fn #method_name(self) -> Option<#variant_name> {
                    match self {
                        #name::#variant_name(v) => Some(v),
                        _ => None,
                    }
                }
            }
        });

        as_ref_methods.push({
            let method_name = format_ident!("{}_as_ref", method_name, span = variant_name.span());

            quote! {
                pub fn #method_name(&self) -> Option<&#variant_name> {
                    match self {
                        #name::#variant_name(v) => Some(v),
                        _ => None,
                    }
                }
            }
        });

        as_mut_methods.push({
            let method_name = format_ident!("{}_as_mut", method_name, span = variant_name.span());

            quote! {
                pub fn #method_name(&mut self) -> Option<&mut #variant_name> {
                    match self {
                        #name::#variant_name(v) => Some(v),
                        _ => None,
                    }
                }
            }
        });

        from_methods.push(quote! {
            impl From<#variant_name> for #name {
                fn from(v: #variant_name) -> Self {
                    #name::#variant_name(v)
                }
            }

            impl TryFrom<#name> for #variant_name {
                type Error = ();

                fn try_from(v: #name) -> Result<Self, Self::Error> {
                    match v {
                        #name::#variant_name(v) => Ok(v),
                        _ => Err(()),
                    }
                }
            }
        });

        serialize_entries.push(quote! {
            #name::#variant_name(v) => {
                borsh::BorshSerialize::serialize(v, writer)?;
            }
        });

        variant_consts.push(quote! {
            pub const #const_name: u8 = #discriminant_name::#variant_name.code();
        });

        deserialize_entries.push(quote! {
            #const_name => {
                let v = borsh::BorshDeserialize::deserialize(buf)?;
                Ok(#name::#variant_name(v))
            }
        });

        discriminants_as_list.push(quote! {
            #discriminant_name::#variant_name.code()
        });

        // Insert discriminant.
        used_discriminants.insert(const_name);
    }

    let derive_enum_discriminants = if arguments.accounts_type_name.is_none() {
        quote! {
            #[derive(EnumDiscriminants)]
        }
    } else {
        quote! {}
    };

    let enum_discriminants = if let Some(accounts_type_name) = &arguments.accounts_type_name {
        let mut fields = Vec::new();
        let mut codes = Vec::new();
        let mut discriminants = Vec::new();
        let accounts_type_discriminant_name = format_ident!("{}Discriminant", accounts_type_name);

        for variant in &variants {
            let variant_ident = &variant.name;

            fields.push(quote! {
                #variant_ident
            });

            discriminants.push(quote!(
                Self::#variant_ident{..} => #discriminant_name::#variant_ident
            ));

            codes.push(quote!(
                Self::#variant_ident => #accounts_type_discriminant_name::#variant_ident.code()
            ));
        }

        quote! {
            #[allow(dead_code)]
            #[automatically_derived]
            #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
            #[non_exhaustive]
            #[repr(u8)]
            #visibility enum #discriminant_name {
                #(#fields,)*
            }

            #[automatically_derived]
            impl #discriminant_name {
                #[inline(always)]
                pub const fn code(&self) -> u8 {
                    match self {
                        #(#codes,)*
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics #name #ty_generics #where_clause {
                #[inline(always)]
                pub const fn discriminant(&self) -> #discriminant_name {
                    match self {
                        #(#discriminants,)*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    let const_asserts = if arguments.accounts_type_name.is_some() {
        discriminants_as_list
            .windows(2)
            .map(|v| {
                let prev = &v[0];
                let next = &v[1];

                quote! {
                    const_assert!(#prev < #next);
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let result = quote! {
        #(#const_asserts)*

        #(#attrs)*
        #derive_enum_discriminants
        #[derive(FankorSerialize, FankorDeserialize, TsGen)]
        #[non_exhaustive]
        #[repr(u8)]
        #visibility enum #name #ty_generics #where_clause {
            #(#final_enum_variants,)*
        }

        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#unwrap_methods)*

            #(#as_ref_methods)*

            #(#as_mut_methods)*
        }

        #(#from_methods)*

        #[automatically_derived]
        impl #impl_generics ::fankor::traits::AccountSerialize for #name #ty_generics #where_clause {
            fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> ::fankor::errors::FankorResult<()> {
                if ::fankor::prelude::borsh::BorshSerialize::serialize(self, writer).is_err() {
                    return Err(::fankor::errors::FankorErrorCode::AccountDidNotSerialize {
                        account: #name_str.to_string()
                    }.into());
                }
                Ok(())
            }
        }

        #[automatically_derived]
        impl #impl_generics ::fankor::traits::AccountDeserialize for #name #ty_generics #where_clause {
            fn try_deserialize(buf: &mut &[u8]) -> ::fankor::errors::FankorResult<Self> {
                unsafe { Self::try_deserialize_unchecked(buf) }
            }

            unsafe fn try_deserialize_unchecked(buf: &mut &[u8]) -> ::fankor::errors::FankorResult<Self> {
                ::fankor::prelude::borsh::BorshDeserialize::deserialize(buf)
                    .map_err(|_| ::fankor::errors::FankorErrorCode::AccountDidNotDeserialize {
                    account: #name_str.to_string()
                }.into())
            }
        }

        #[automatically_derived]
        impl #impl_generics ::fankor::traits::AccountType for #name #ty_generics #where_clause {
            fn discriminant() -> u8 {
                0
            }

            fn owner() -> &'static Pubkey {
                &crate::ID
            }

            fn check_discriminant(discriminant: u8) -> bool {
                const discriminants: &[u8] = &[#(#discriminants_as_list),*];
                discriminants.binary_search(&discriminant).is_ok()
            }
        }

        #enum_discriminants
    };

    Ok(result.into())
}
