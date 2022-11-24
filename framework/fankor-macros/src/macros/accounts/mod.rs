use convert_case::{Boundary, Case, Converter};
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, Item};

use crate::Result;

use crate::macros::accounts::arguments::AccountsArguments;
use crate::macros::accounts::variant::AccountVariant;

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

    // Parse fields taking into account whether any variant is deprecated or not.
    let mut last_deprecated = false;
    let variants = enum_item
        .variants
        .into_iter()
        .map(|v| {
            let result = AccountVariant::from(v)?;

            if last_deprecated && result.discriminant.is_none() {
                return Err(Error::new(
                    result.name.span(),
                    "The next error after a deprecated one must have the #[discriminant] attribute",
                ));
            }

            last_deprecated = result.deprecated;

            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;

    let visibility = enum_item.vis;

    let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();

    // Generate code.
    let mut u8_index = 1u8;
    let mut used_discriminants = HashSet::new();

    let mut final_enum_variants = Vec::with_capacity(variants.len());
    let mut unwrap_methods = Vec::with_capacity(variants.len());
    let mut as_ref_methods = Vec::with_capacity(variants.len());
    let mut as_mut_methods = Vec::with_capacity(variants.len());
    let mut discriminant_variants = Vec::with_capacity(variants.len());
    let mut discriminant_code_variants = Vec::with_capacity(variants.len());
    let mut final_enum_variant_discriminants = Vec::with_capacity(variants.len());
    let mut from_methods = Vec::with_capacity(variants.len());
    let mut serialize_entries = Vec::with_capacity(variants.len());
    let mut deserialize_entries = Vec::with_capacity(variants.len());
    let code_variants = variants
        .iter()
        .map(|v| {
            let span = v.name.span();
            let AccountVariant {
                name: variant_name,
                attributes,
                discriminant,
                ..
            } = &v;

            let case_converter = Converter::new()
                .from_case(Case::Pascal)
                .to_case(Case::Snake)
                .remove_boundary(Boundary::LowerDigit)
                .remove_boundary(Boundary::UpperDigit);

            final_enum_variants.push(quote! {
                #(#attributes)*
                #variant_name(#variant_name)
            });

            let method_name = case_converter.convert(&variant_name.to_string());

            unwrap_methods.push({
                let method_name =
                    format_ident!("unwrap_{}", method_name, span = variant_name.span());

                quote! {
                    fn #method_name(self) -> Option<#variant_name> {
                        match self {
                            #name::#variant_name(v) => Some(v),
                            _ => None,
                        }
                    }
                }
            });

            as_ref_methods.push({
                let method_name =
                    format_ident!("{}_as_ref", method_name, span = variant_name.span());

                quote! {
                    fn #method_name(&self) -> Option<&#variant_name> {
                        match self {
                            #name::#variant_name(v) => Some(v),
                            _ => None,
                        }
                    }
                }
            });

            as_mut_methods.push({
                let method_name =
                    format_ident!("{}_as_mut", method_name, span = variant_name.span());

                quote! {
                    fn #method_name(&mut self) -> Option<&mut #variant_name> {
                        match self {
                            #name::#variant_name(v) => Some(v),
                            _ => None,
                        }
                    }
                }
            });

            discriminant_variants.push(quote! {
                #name::#variant_name(..) => #discriminant_name::#variant_name
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

            // Calculate the discriminator.
            if let Some(v) = discriminant {
                let new_value = v.base10_parse::<u8>()?;

                u8_index = new_value;
            }

            if u8_index == 0 {
                return Err(Error::new(
                    span,
                    "Zero discriminant is reserved for uninitialized accounts, please provide another one".to_string(),
                ));
            }

            if used_discriminants.contains(&u8_index) {
                return Err(Error::new(
                    span,
                    format!("The discriminant attribute is already in use: {}", u8_index),
                ));
            }

            used_discriminants.insert(u8_index);

            if arguments.contains_removed_discriminant(u8_index) {
                return Err(Error::new(
                    name.span(),
                    format!("The discriminator '{}' is marked as removed", u8_index),
                ));
            }

            discriminant_code_variants.push(quote! {
                #name::#variant_name(..) => #u8_index
            });

            final_enum_variant_discriminants.push(quote! {
                #variant_name
            });

            serialize_entries.push(quote! {
                #name::#variant_name(v) => {
                    borsh::BorshSerialize::serialize(&#u8_index, writer)?;
                    borsh::BorshSerialize::serialize(v, writer)?;
                }
            });

            deserialize_entries.push(quote! {
                #u8_index => {
                    let v = borsh::BorshDeserialize::deserialize(buf)?;
                    Ok(#name::#variant_name(v))
                }
            });

            let res = Ok(quote! {
                #discriminant_name::#variant_name => #u8_index
            });

            u8_index += 1;

            res
        })
        .collect::<Result<Vec<_>>>()?;

    let result = quote! {
        #(#attrs)*
        #visibility enum #name #ty_generics #where_clause {
            #(#final_enum_variants,)*
        }

        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#unwrap_methods)*

            #(#as_ref_methods)*

            #(#as_mut_methods)*

            pub fn discriminant(&self) -> #discriminant_name {
                match self {
                    #(#discriminant_variants,)*
                }
            }

            pub fn discriminant_code(&self) -> u8 {
                match self {
                    #(#discriminant_code_variants,)*
                }
            }
        }

        #(#from_methods)*

        impl #impl_generics borsh::BorshSerialize for #name #ty_generics #where_clause {
            fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                match self {
                    #(#serialize_entries)*
                }

                Ok(())
            }
        }

        impl #impl_generics borsh::BorshDeserialize for #name #ty_generics #where_clause {
            fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                let discriminant = borsh::BorshDeserialize::deserialize(buf)?;

                match discriminant {
                    #(#deserialize_entries)*
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid discriminant value",
                    )),
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics ::fankor::traits::AccountSerialize for #name #ty_generics #where_clause {
            fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> ::fankor::errors::FankorResult<()> {
                if writer.write_all(&[<#name #ty_generics as ::fankor::traits::Account>::discriminator()]).is_err() {
                    return Err(::fankor::errors::FankorErrorCode::AccountDidNotSerialize{
                        account: #name_str.to_string()
                    }.into());
                }

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
        impl #impl_generics ::fankor::traits::Account for #name #ty_generics #where_clause {
            fn discriminator() -> u8 {
                0
            }

            fn owner() -> &'static Pubkey {
                &crate::ID
            }
        }

        #visibility enum #discriminant_name {
            #(#final_enum_variant_discriminants,)*
        }

        #[automatically_derived]
        impl #discriminant_name {
            pub fn code(&self) -> u8 {
                match self {
                    #(#code_variants,)*
                }
            }
        }
    };

    Ok(result.into())
}
