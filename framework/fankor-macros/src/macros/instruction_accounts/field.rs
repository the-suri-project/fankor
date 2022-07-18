use crate::macros::instruction_accounts::parser::CustomMetaList;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Fields, GenericArgument, PathArguments, Type, Variant};

use crate::Result;

pub struct Field {
    pub name: Ident,
    pub ty: Type,
    pub kind: FieldKind,
    // Attributes.
    pub owner: Option<TokenStream>,
    pub address: Option<TokenStream>,
    pub initialized: Option<TokenStream>,
    pub writable: Option<TokenStream>,
    pub executable: Option<TokenStream>,
    pub rent_exempt: Option<TokenStream>,
    pub signer: Option<TokenStream>,
    pub min: Option<TokenStream>,
    pub max: Option<TokenStream>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FieldKind {
    Other,
    Vec(Type),
    Rest,
}

impl Field {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the Field struct from the given attributes.
    pub fn from(field: syn::Field) -> Result<Field> {
        let mut new_field = Field {
            name: field.ident.unwrap(),
            kind: discriminate_type(&field.ty),
            ty: field.ty,
            owner: None,
            address: None,
            initialized: None,
            writable: None,
            executable: None,
            rent_exempt: None,
            signer: None,
            min: None,
            max: None,
        };

        new_field.parse_attributes(field.attrs)?;

        Ok(new_field)
    }

    /// Creates a new instance of the Field struct from the given attributes.
    pub fn from_variant(variant: Variant) -> Result<Field> {
        match variant.fields {
            Fields::Unnamed(v) => {
                if v.unnamed.len() != 1 {
                    return Err(Error::new(
                        v.span(),
                        "Instruction variants can only have a single unnamed field, i.e. Variant(<account>)",
                    ));
                }

                let ty = v.unnamed.first().unwrap().ty.clone();
                let mut new_field = Field {
                    name: variant.ident,
                    kind: discriminate_type(&ty),
                    ty,
                    owner: None,
                    address: None,
                    initialized: None,
                    writable: None,
                    executable: None,
                    rent_exempt: None,
                    signer: None,
                    min: None,
                    max: None,
                };

                new_field.parse_attributes(variant.attrs)?;

                Ok(new_field)
            }
            _ => Err(Error::new(
                variant.span(),
                "Instruction variants must be like: Variant(<account>)",
            )),
        }
    }

    fn parse_attributes(&mut self, mut attrs: Vec<Attribute>) -> Result<()> {
        let mut size_attr = false;

        while let Some(attribute) = attrs.pop() {
            if !attribute.path.is_ident("account") {
                continue;
            }

            let attribute_span = attribute.span();
            let args = match attribute.parse_args::<CustomMetaList>() {
                Ok(v) => v,
                Err(_) => {
                    return Err(Error::new(
                        attribute_span,
                        "The account attribute expects arguments",
                    ));
                }
            };

            // Check each argument.
            for meta in args.list {
                let name = meta.name;
                if let Some(value) = meta.value {
                    match name.to_string().as_str() {
                        "owner" => {
                            if self.owner.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The owner argument can only be defined once",
                                ));
                            }

                            self.owner = Some(quote! {#value});
                        }
                        "address" => {
                            if self.address.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The address argument can only be defined once",
                                ));
                            }

                            self.address = Some(quote! {#value});
                        }
                        "initialized" => {
                            if self.initialized.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The initialized argument can only be defined once",
                                ));
                            }

                            self.initialized = Some(quote! {#value});
                        }
                        "writable" => {
                            if self.writable.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The writable argument can only be defined once",
                                ));
                            }

                            self.writable = Some(quote! {#value});
                        }
                        "executable" => {
                            if self.executable.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The executable argument can only be defined once",
                                ));
                            }

                            self.executable = Some(quote! {#value});
                        }
                        "rent_exempt" => {
                            if self.rent_exempt.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The rent_exempt argument can only be defined once",
                                ));
                            }

                            self.rent_exempt = Some(quote! {#value});
                        }
                        "signer" => {
                            if self.signer.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The signer argument can only be defined once",
                                ));
                            }

                            self.signer = Some(quote! {#value});
                        }
                        "min" => {
                            if size_attr {
                                return Err(Error::new(
                                    name.span(),
                                    "The min argument is incompatible with the size argument",
                                ));
                            }

                            if self.min.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The min argument can only be defined once",
                                ));
                            }

                            self.min = Some(quote! {#value});
                        }
                        "max" => {
                            if size_attr {
                                return Err(Error::new(
                                    name.span(),
                                    "The max argument is incompatible with the size argument",
                                ));
                            }

                            if self.max.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The max argument can only be defined once",
                                ));
                            }

                            self.max = Some(quote! {#value});
                        }
                        "size" => {
                            if size_attr {
                                return Err(Error::new(
                                    name.span(),
                                    "The size argument can only be defined once",
                                ));
                            }

                            if self.min.is_some() || self.max.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The size argument is incompatible with the min and max arguments",
                                ));
                            }

                            self.min = Some(quote! {#value});
                            self.max = Some(quote! {#value});
                            size_attr = true;
                        }
                        _ => {
                            return Err(Error::new(name.span(), "Unknown argument"));
                        }
                    }
                } else {
                    match name.to_string().as_str() {
                        "owner" => {
                            return Err(Error::new(
                                name.span(),
                                "The owner argument must use a value: owner = <expr>",
                            ));
                        }
                        "address" => {
                            return Err(Error::new(
                                name.span(),
                                "The address argument must use a value: address = <expr>",
                            ));
                        }
                        "initialized" => {
                            if self.initialized.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The initialized argument can only be defined once",
                                ));
                            }

                            self.initialized = Some(quote! {true});
                        }
                        "writable" => {
                            if self.writable.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The writable argument can only be defined once",
                                ));
                            }

                            self.writable = Some(quote! {true});
                        }
                        "executable" => {
                            if self.executable.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The executable argument can only be defined once",
                                ));
                            }

                            self.executable = Some(quote! {true});
                        }
                        "rent_exempt" => {
                            if self.rent_exempt.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The rent_exempt argument can only be defined once",
                                ));
                            }

                            self.rent_exempt = Some(quote! {true});
                        }
                        "signer" => {
                            if self.signer.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The signer argument can only be defined once",
                                ));
                            }

                            self.signer = Some(quote! {true});
                        }
                        "min" => {
                            return Err(Error::new(
                                name.span(),
                                "The min argument must use a value: min = <expr>",
                            ));
                        }
                        "max" => {
                            return Err(Error::new(
                                name.span(),
                                "The max argument must use a value: max = <expr>",
                            ));
                        }
                        "size" => {
                            return Err(Error::new(
                                name.span(),
                                "The size argument must use a value: size = <expr>",
                            ));
                        }
                        _ => {
                            return Err(Error::new(name.span(), "Unknown argument"));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

fn discriminate_type(ty: &Type) -> FieldKind {
    if let Type::Path(v) = ty {
        let last_arg = v.path.segments.last().unwrap();
        if &last_arg.ident.to_string() == "Vec" {
            return match &last_arg.arguments {
                PathArguments::AngleBracketed(v) => {
                    let first = v.args.first().unwrap();
                    match first {
                        GenericArgument::Type(v) => FieldKind::Vec(v.clone()),
                        _ => FieldKind::Other,
                    }
                }
                _ => FieldKind::Other,
            };
        }

        if &last_arg.ident.to_string() == "Rest" {
            return FieldKind::Rest;
        }
    }

    FieldKind::Other
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub fn check_fields(fields: &[Field]) -> Result<()> {
    let mut rest_field = false;
    for field in fields {
        match &field.kind {
            FieldKind::Other => {
                if rest_field {
                    return Err(Error::new(
                        field.name.span(),
                        "The rest field cannot be placed after other fields",
                    ));
                }

                if field.min.is_some() || field.max.is_some() {
                    return Err(Error::new(
                        field.name.span(),
                        "The min, max and size attributes are compatible only with Vec and Rest",
                    ));
                }
            }
            FieldKind::Vec(_) => {
                if rest_field {
                    return Err(Error::new(
                        field.name.span(),
                        "The rest field cannot be placed after other fields",
                    ));
                }
            }
            FieldKind::Rest => {
                if rest_field {
                    return Err(Error::new(
                        field.name.span(),
                        "The rest field can only be defined once",
                    ));
                }

                rest_field = true;
            }
        }
    }

    Ok(())
}
