use crate::Result;
use convert_case::{Case, Converter};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Fields, Item};

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    let case_converter = Converter::new()
        .from_case(Case::Snake)
        .to_case(Case::Pascal);

    // Process input.
    let (result, account_name) = match &input {
        Item::Struct(item) => {
            let name = &item.ident;
            let name_str = name.to_string();
            let generics = &item.generics;
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

            let schema_name = format!("{}Schema", name_str);
            let schema_constant_name = format!("T{}", name_str);

            let mut constructor_replacements = Vec::new();
            let mut schema_replacements = Vec::new();
            let mut ts_constructor_params = Vec::new();
            let mut ts_schema_constructor_args = Vec::new();
            let mut ts_schema_fields = Vec::new();

            for field in &item.fields {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = case_converter.convert(field_name.to_string());
                let field_name = format_ident!("{}", field_name_str, span = field_name.span());
                let field_ty = &field.ty;

                let constructor_replacement_str = format!("_r_constructor_{}_r_", field_name);
                let schema_replacement_str = format!("_r_schema_{}_r_", field_name);
                ts_constructor_params.push(format!(
                    "public {}: {}",
                    field_name, constructor_replacement_str
                ));
                ts_schema_constructor_args.push(format!("result.{}", field_name));
                ts_schema_fields.push(format!("['{}', {}]", field_name, schema_replacement_str));

                constructor_replacements.push(quote! {
                    .replace(#constructor_replacement_str, &< #field_ty as TsTypeGen>::generate_type(registered_types))
                });

                schema_replacements.push(quote! {
                    .replace(#schema_replacement_str, &< #field_ty as TsTypeGen>::generate_schema(registered_schemas))
                });
            }

            let ts_type = format!(
                "export class {} {{
                    // CONSTRUCTORS -----------------------------------------------------------

                    constructor({}) {{}}

                    // GETTERS ----------------------------------------------------------------

                    static get schema() {{
                        return {};
                    }}

                    // METHODS ----------------------------------------------------------------

                    serialize(buffer?: Buffer) {{
                        const writer = new fnk.FnkBorshWriter(buffer);
                        {}.serialize(writer, this);
                        return writer.toByteArray();
                    }}

                    // STATIC METHODS ---------------------------------------------------------

                    static deserialize(buffer: Buffer, offset?: number) {{
                        const reader = new fnk.FnkBorshReader(buffer, offset);
                        return {}.deserialize(reader);
                    }}
                }}",
                name_str,
                ts_constructor_params.join(","),
                schema_constant_name,
                schema_constant_name,
                schema_constant_name,
            );

            let ts_schema = format!(
                "export class {} implements fnk.FnkBorshSchema<{}> {{
                    innerSchema = fnk.TStruct([{}] as const);

                    // METHODS ----------------------------------------------------------------

                    serialize(writer: fnk.FnkBorshWriter, value: {}) {{
                        this.innerSchema.serialize(writer, value);
                    }}

                    deserialize(reader: fnk.FnkBorshReader) {{
                        const result = this.innerSchema.deserialize(reader);
                        return new {}({});
                    }}
                }}

                export const {} = new {}();
                ",
                schema_name,
                name_str,
                ts_schema_fields.join(","),
                name_str,
                name_str,
                ts_schema_constructor_args.join(","),
                schema_constant_name,
                schema_name,
            );

            let result = quote! {
                #[automatically_derived]
                impl #impl_generics ::fankor::prelude::ts_gen::types::TsTypeGen for #name #ty_generics #where_clause {
                    fn value(&self) -> std::borrow::Cow<'static, str> {
                        unreachable!()
                    }

                    fn value_type() -> std::borrow::Cow<'static, str> {
                        std::borrow::Cow::Borrowed(#name_str)
                    }

                    fn schema_name() -> std::borrow::Cow<'static, str> {
                        std::borrow::Cow::Borrowed(#schema_constant_name)
                    }

                    fn generate_type(registered_types: &mut ::fankor::prelude::ts_gen::types::TsTypesCache) -> std::borrow::Cow<'static, str> {
                        use ::fankor::prelude::ts_gen::types::TsTypeGen;
                        let name = Self::value_type();

                        if registered_types.contains_key(&name) {
                            return name;
                        }

                        let ts_type = #ts_type #(#constructor_replacements)*;
                        registered_types.insert(name.clone(), std::borrow::Cow::Owned(ts_type));

                        name
                    }

                    fn generate_schema(registered_schemas: &mut ::fankor::prelude::ts_gen::types::TsTypesCache) -> std::borrow::Cow<'static, str> {
                        use ::fankor::prelude::ts_gen::types::TsTypeGen;
                        let name = Self::schema_name();

                        if registered_schemas.contains_key(&name) {
                            return name;
                        }

                        let ts_schema = #ts_schema #(#schema_replacements)*;
                        registered_schemas.insert(name.clone(), std::borrow::Cow::Owned(ts_schema));

                        name
                    }
                }
            };

            (result, name)
        }
        Item::Enum(item) => {
            let name = &item.ident;
            let name_str = name.to_string();
            let generics = &item.generics;
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

            let types_name = format!("{}Types", name_str);
            let schema_name = format!("{}Schema", name_str);
            let discriminant_name = format_ident!("{}Discriminant", name);
            let schema_constant_name = format!("T{}", name_str);

            let mut type_replacements = Vec::new();
            let mut schema_replacements = Vec::new();
            let mut ts_schema_fields = Vec::new();
            let mut ts_interface_names = Vec::new();
            let mut ts_interfaces = Vec::new();

            for variant in &item.variants {
                let variant_name = &variant.ident;
                let variant_name_str = variant_name.to_string();
                let variant_name =
                    format_ident!("{}", variant_name_str, span = variant_name.span());
                let interface_name =
                    format_ident!("{}_{}", name, variant_name_str, span = variant_name.span());

                let schema_discriminant_replacement_str =
                    format!("_r_schema_discriminant_{}_r_", variant_name);

                ts_interface_names.push(interface_name.to_string());

                schema_replacements.push(quote! {
                    .replace(#schema_discriminant_replacement_str, &#discriminant_name::#variant_name.code().to_string())
                });

                match &variant.fields {
                    Fields::Named(v) => {
                        let fields = v.named.iter().map(|f| {
                            let field_name = f.ident.as_ref().unwrap();
                            let replacement_str = format!("_r_interface_{}_{}_r_", interface_name, field_name);
                            let ty = &f.ty;

                            type_replacements.push(quote! {
                                .replace(#replacement_str, &< #ty as TsTypeGen>::generate_type(registered_types))
                            });

                            format!(
                                "{}: {}",
                                field_name,
                                replacement_str,
                            )
                        }).collect::<Vec<_>>();

                        ts_interfaces.push(format!(
                            "export interface {} {{ type: '{}'; value: {{ {} }} }}",
                            interface_name,
                            variant_name_str,
                            fields.join(", ")
                        ));

                        let field_schemas = v.named.iter().map(|f| {
                            let field_name = f.ident.as_ref().unwrap();
                            let replacement_str = format!("_r_schema_{}_{}_r_", interface_name, field_name);
                            let ty = &f.ty;

                            schema_replacements.push(quote! {
                                .replace(#replacement_str, &< #ty as TsTypeGen>::generate_schema(registered_schemas))
                            });

                            format!("['{}', {}]", field_name, replacement_str)
                        }).collect::<Vec<_>>();

                        ts_schema_fields.push(format!(
                            "[{},'{}',fnk.TStruct([{}] as const)]",
                            schema_discriminant_replacement_str,
                            variant_name,
                            field_schemas.join(",")
                        ));
                    }
                    Fields::Unnamed(v) => {
                        if v.unnamed.len() != 1 {
                            return Err(Error::new(
                                v.span(),
                                "Only single field unnamed enums are supported",
                            ));
                        }

                        let field = v.unnamed.first().unwrap();
                        let field_ty = &field.ty;
                        let field_replacement_str = format!("_r_interface_{}_r_", interface_name);

                        type_replacements.push(quote! {
                            .replace(#field_replacement_str, &< #field_ty as TsTypeGen>::generate_type(registered_types))
                        });

                        ts_interfaces.push(format!(
                            "export interface {} {{ type: '{}'; value: {} }}",
                            interface_name, variant_name_str, field_replacement_str
                        ));

                        let schema_replacement_str = format!("_r_schema_{}_r_", interface_name);

                        schema_replacements.push(quote! {
                            .replace(#schema_replacement_str, &< #field_ty as TsTypeGen>::generate_schema(registered_schemas))
                        });

                        ts_schema_fields.push(format!(
                            "[{},'{}',{}]",
                            schema_discriminant_replacement_str,
                            variant_name,
                            schema_replacement_str
                        ));
                    }
                    Fields::Unit => {
                        ts_interfaces.push(format!(
                            "export interface {} {{ type: '{}'; value: null; }}",
                            interface_name, variant_name_str,
                        ));

                        ts_schema_fields.push(format!(
                            "[{},'{}',fnk.Unit]",
                            schema_discriminant_replacement_str, variant_name
                        ));
                    }
                }
            }

            let ts_type = format!(
                "export class {} {{
                    // CONSTRUCTORS -----------------------------------------------------------

                    constructor(public data: {}) {{}}

                    // GETTERS ----------------------------------------------------------------

                    static get schema() {{
                        return {};
                    }}

                    get type() {{
                        return this.data.type;
                    }}

                    get schema() {{
                        return this.data.value;
                    }}

                    // METHODS ----------------------------------------------------------------

                    serialize(buffer?: Buffer) {{
                        const writer = new fnk.FnkBorshWriter(buffer);
                        {}.serialize(writer, this);
                        return writer.toByteArray();
                    }}

                    // STATIC METHODS ---------------------------------------------------------

                    static deserialize(buffer: Buffer, offset?: number) {{
                        const reader = new fnk.FnkBorshReader(buffer, offset);
                        return {}.deserialize(reader);
                    }}
                }}

                export type {} = {};

                {}",
                name_str,
                types_name,
                schema_constant_name,
                schema_constant_name,
                schema_constant_name,
                types_name,
                ts_interface_names.join("|"),
                ts_interfaces.join("\n")
            );

            let ts_schema = format!(
                "export class {} implements fnk.FnkBorshSchema<{}> {{
                    innerSchema = fnk.TEnum([{}] as const);

                    // METHODS ----------------------------------------------------------------

                    serialize(writer: fnk.FnkBorshWriter, value: {}) {{
                        this.innerSchema.serialize(writer, value);
                    }}

                    deserialize(reader: fnk.FnkBorshReader) {{
                        const result = this.innerSchema.deserialize(reader);
                        return new {}(result.value);
                    }}
                }}

                export const {} = new {}();
                ",
                schema_name,
                name_str,
                ts_schema_fields.join(","),
                name_str,
                name_str,
                schema_constant_name,
                schema_name,
            );

            let result = quote! {
                #[automatically_derived]
                impl #impl_generics ::fankor::prelude::ts_gen::types::TsTypeGen for #name #ty_generics #where_clause {
                    fn value(&self) -> std::borrow::Cow<'static, str> {
                        unreachable!()
                    }

                    fn value_type() -> std::borrow::Cow<'static, str> {
                        std::borrow::Cow::Borrowed(#name_str)
                    }

                    fn schema_name() -> std::borrow::Cow<'static, str> {
                        std::borrow::Cow::Borrowed(#schema_constant_name)
                    }

                    fn generate_type(registered_types: &mut ::fankor::prelude::ts_gen::types::TsTypesCache) -> std::borrow::Cow<'static, str> {
                        use ::fankor::prelude::ts_gen::types::TsTypeGen;
                        let name = Self::value_type();

                        if registered_types.contains_key(&name) {
                            return name;
                        }

                        let ts_type = #ts_type #(#type_replacements)*;
                        registered_types.insert(name.clone(), std::borrow::Cow::Owned(ts_type));

                        name
                    }

                    fn generate_schema(registered_schemas: &mut ::fankor::prelude::ts_gen::types::TsTypesCache) -> std::borrow::Cow<'static, str> {
                        use ::fankor::prelude::ts_gen::types::TsTypeGen;
                        let name = Self::schema_name();

                        if registered_schemas.contains_key(&name) {
                            return name;
                        }

                        let ts_schema = #ts_schema #(#schema_replacements)*;
                        registered_schemas.insert(name.clone(), std::borrow::Cow::Owned(ts_schema));

                        name
                    }
                }
            };

            (result, name)
        }
        _ => {
            unreachable!()
        }
    };

    let account_name_str = account_name.to_string();
    let test_name = format_ident!("__ts_gen_test__account_{}", account_name_str);
    let test_name_str = test_name.to_string();
    let result = quote! {
        #[cfg(all(test, feature = "ts-gen"))]
        #[automatically_derived]
        #[allow(non_snake_case)]
        pub mod #test_name {
            use super::*;

            #result

            #[test]
            fn build() {
                 // Register action.
                crate::__ts_gen_test__setup::BUILD_CONTEXT.register_action(#test_name_str, file!(), move |action_context| {
                    action_context.add_account::<#account_name>().unwrap();
                })
            }
        }
    };

    Ok(result.into())
}
