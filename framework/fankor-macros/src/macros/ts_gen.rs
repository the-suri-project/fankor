use crate::fnk_syn::FnkMetaArgumentList;
use crate::Result;
use convert_case::{Case, Converter};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Fields, Item};

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    let case_converter = Converter::new().from_case(Case::Snake).to_case(Case::Camel);

    // Process input.
    let (result, account_name) = match &input {
        Item::Struct(item) => {
            let name = &item.ident;
            let name_str = name.to_string();
            let generics = &item.generics;
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

            // Check for ts_gen attribute.
            let mut account_discriminants = None;

            for attr in &item.attrs {
                if attr.path.is_ident("fankor") {
                    if let Ok(mut args) = attr.parse_args::<FnkMetaArgumentList>() {
                        args.error_on_duplicated()?;

                        account_discriminants = args.pop_ident("account", true)?;

                        if args.pop_plain("accounts", true)? {
                            return Err(Error::new(
                                input.span(),
                                "Accounts cannot be used with an struct",
                            ));
                        }

                        args.error_on_unknown()?;
                    } else {
                        return Err(Error::new(
                            attr.span(),
                            "The correct pattern is #[ts_gen(<meta_list>)]",
                        ));
                    };
                    break;
                }
            }

            let schema_name = format!("{}Schema", name_str);
            let schema_constant_name = format!("T{}", name_str);

            let mut ts_replacements = Vec::new();
            let mut schema_replacements = Vec::new();
            let mut ts_fields = String::new();
            let mut ts_optional_field = String::new();
            let mut ts_constructor_fields = String::new();
            let mut ts_schema_fields = Vec::new();
            let mut equals_method_conditions = Vec::new();
            let mut clone_method_fields = Vec::new();

            for field in &item.fields {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = case_converter.convert(field_name.to_string());
                let field_name = format_ident!("{}", field_name_str, span = field_name.span());
                let field_ty = &field.ty;

                let schema_replacement_str = format!("_r_schema_{}_r_", field_name);
                ts_schema_fields.push(format!("['{}', {}]", field_name, schema_replacement_str));

                let ts_field_replacement_format_str = format!("public {}: {{}};", field_name);
                let ts_field_replacement_str = format!("_r_field_{}_r_", field_name);
                ts_fields.push_str(&ts_field_replacement_str);
                ts_replacements.push(quote! {
                    .replace(#ts_field_replacement_str, &format!(#ts_field_replacement_format_str, < #field_ty as TsTypeGen>::generate_type(registered_types)))
                });

                let ts_optional_field_str = format!(" _r_optional_field_{}_r_", field_name);
                let ts_optional_field_replacement_str = format!("| '{}'", field_name);
                ts_optional_field.push_str(&ts_optional_field_str);
                ts_replacements.push(quote! {
                    .replace(#ts_optional_field_str,  if < #field_ty as TsTypeGen>::unit_value().is_some() {
                        #ts_optional_field_replacement_str
                    } else {
                        ""
                    })
                });

                let ts_constructor_field_str = format!(" _r_constructor_field_{}_r_", field_name);
                let ts_constructor_field_replacement_str =
                    format!("this.{} = data.{};", field_name, field_name);
                let ts_constructor_field_optional_replacement_str =
                    format!("this.{} = data.{} ?? {{}};", field_name, field_name);
                ts_constructor_fields.push_str(&ts_constructor_field_str);
                ts_replacements.push(quote! {
                    .replace(#ts_constructor_field_str,  &if let Some(unit_value) = < #field_ty as TsTypeGen>::unit_value() {
                        format!(#ts_constructor_field_optional_replacement_str, unit_value)
                    } else {
                        #ts_constructor_field_replacement_str.to_string()
                    })
                });

                schema_replacements.push(quote! {
                    .replace(#schema_replacement_str, &< #field_ty as TsTypeGen>::generate_schema(registered_schemas))
                });

                equals_method_conditions.push(format!(
                    "fnk.equals(this.{}, other.{})",
                    field_name_str, field_name_str
                ));

                clone_method_fields.push(format!("{}: fnk.clone(this.{})", field_name, field_name));
            }

            let ts_type = format!(
                "export class {} {{
                    // FIELDS -----------------------------------------------------------------
                    {}

                    // CONSTRUCTORS -----------------------------------------------------------

                    constructor(data: fnk.OptionalFields<{}, '' {}>) {{
                        {}
                    }}

                    // GETTERS ----------------------------------------------------------------

                    static get schema() {{
                        return {};
                    }}

                    // METHODS ----------------------------------------------------------------

                    serialize(buffer?: Buffer) {{
                        const writer = new fnk.FnkBorshWriter(buffer);
                        {}.serialize(writer, this);
                        return writer.toBuffer();
                    }}

                    equals(other: {}) {{
                        return {};
                    }}

                    clone(): {} {{
                        return new {}({{ {} }});
                    }}

                    // STATIC METHODS ---------------------------------------------------------

                    static deserialize(buffer: Buffer, offset?: number) {{
                        const reader = new fnk.FnkBorshReader(buffer, offset);
                        return {}.deserialize(reader);
                    }}
                }}",
                name_str,
                ts_fields,
                name_str,
                ts_optional_field,
                ts_constructor_fields,
                schema_constant_name,
                schema_constant_name,
                name_str,
                equals_method_conditions.join(" && "),
                name_str,
                name_str,
                clone_method_fields.join(","),
                schema_constant_name,
            );

            let ts_schema = if let Some(account_discriminants) = account_discriminants {
                format!(
                    "export class {} implements fnk.FnkBorshSchema<{}> {{
                        innerSchema = null as any as ReturnType<{}['initSchema']>;

                        // METHODS ----------------------------------------------------------------

                        initSchema() {{
                            const innerSchema = fnk.TStruct([
                                ['discriminant', fnk.U8],
                                {}
                            ] as const);
                            this.innerSchema = innerSchema;
                            return innerSchema;
                        }}

                        // METHODS ----------------------------------------------------------------

                        serialize(writer: fnk.FnkBorshWriter, value: {}) {{
                            this.innerSchema.serialize(writer, {{
                                discriminant: {}.{},
                                ...value
                            }});
                        }}

                        deserialize(reader: fnk.FnkBorshReader) {{
                            const data:any = this.innerSchema.deserialize(reader);
                            if (data.discriminant !== {}.{}) {{
                                throw new Error('Invalid discriminant');
                            }}
                            return new {}(data);
                        }}
                    }}",
                    schema_name,
                    name_str,
                    schema_name,
                    ts_schema_fields.join(","),
                    name_str,
                    account_discriminants,
                    name_str,
                    account_discriminants,
                    name_str,
                    name_str,
                )
            } else {
                format!(
                    "export class {} implements fnk.FnkBorshSchema<{}> {{
                        innerSchema = null as any as ReturnType<{}['initSchema']>;

                        // METHODS ----------------------------------------------------------------

                        initSchema() {{
                            const innerSchema = fnk.TStruct([{}] as const);
                            this.innerSchema = innerSchema;
                            return innerSchema;
                        }}

                        // METHODS ----------------------------------------------------------------

                        serialize(writer: fnk.FnkBorshWriter, value: {}) {{
                            this.innerSchema.serialize(writer, value);
                        }}

                        deserialize(reader: fnk.FnkBorshReader) {{
                            const data:any = this.innerSchema.deserialize(reader);
                            return new {}(data);
                        }}
                    }}",
                    schema_name,
                    name_str,
                    schema_name,
                    ts_schema_fields.join(","),
                    name_str,
                    name_str,
                )
            };

            let ts_schema_use_method_name = format!("use{}", schema_name);
            let ts_schema_use_method_call = format!("{}()", ts_schema_use_method_name);
            let ts_schema_constant = format!(
                "export const {} = {};",
                schema_constant_name, ts_schema_use_method_call,
            );

            let ts_schema_use_method = format!(
                "const {} = (() => {{
                    let variable: {} | null = null;
                    return () => {{
                        if (variable === null) {{
                            variable = new {}();
                            variable.initSchema();
                        }}

                        return variable
                    }};
                }})();",
                ts_schema_use_method_name, schema_name, schema_name,
            );

            let result = quote! {
                #[automatically_derived]
                impl #impl_generics ::fankor::prelude::TsTypeGen for #name #ty_generics #where_clause {
                    fn value(&self) -> std::borrow::Cow<'static, str> {
                        unreachable!()
                    }

                    fn value_type() -> std::borrow::Cow<'static, str> {
                        std::borrow::Cow::Borrowed(#name_str)
                    }

                    fn schema_name() -> std::borrow::Cow<'static, str> {
                        std::borrow::Cow::Borrowed(#ts_schema_use_method_call)
                    }

                    fn generate_type(registered_types: &mut ::fankor::prelude::TsTypesCache) -> std::borrow::Cow<'static, str> {
                        use ::fankor::prelude::TsTypeGen;
                        let name = Self::value_type();

                        if registered_types.contains_key(&name) {
                            return name;
                        }

                        // Prevents infinite recursion.
                        registered_types.insert(name.clone(), std::borrow::Cow::Borrowed(""));

                        let ts_type = #ts_type.to_string() #(#ts_replacements)*;
                        *registered_types.get_mut(&name).unwrap() = std::borrow::Cow::Owned(ts_type);

                        name
                    }

                    fn generate_schema(registered_schemas: &mut ::fankor::prelude::TsTypesCache) -> std::borrow::Cow<'static, str> {
                        use ::fankor::prelude::TsTypeGen;
                        let name = Self::schema_name();

                        if registered_schemas.contains_key(&name) {
                            return name;
                        }

                        // Prevents infinite recursion.
                        registered_schemas.insert(name.clone(), std::borrow::Cow::Borrowed(""));

                        let ts_schema = #ts_schema.to_string() #(#schema_replacements)*;
                        *registered_schemas.get_mut(&name).unwrap() = std::borrow::Cow::Owned(ts_schema);

                        name
                    }

                    fn generate_schema_constant(registered_constants: &mut ::fankor::prelude::TsTypesCache) {
                        use ::fankor::prelude::TsTypeGen;
                        let name = Self::schema_name();

                        if registered_constants.contains_key(&name) {
                            return;
                        }

                        let ts_schema = #ts_schema_constant .to_string();
                        registered_constants.insert(name.clone(), std::borrow::Cow::Owned(ts_schema));
                    }

                    fn generate_schema_use_method(registered_use_methods: &mut ::fankor::prelude::TsTypesCache) {
                        use ::fankor::prelude::TsTypeGen;
                        let name = Self::schema_name();

                        if registered_use_methods.contains_key(&name) {
                            return;
                        }

                        let ts_schema = #ts_schema_use_method .to_string();
                        registered_use_methods.insert(name.clone(), std::borrow::Cow::Owned(ts_schema));
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

            // Check for ts_gen attribute.
            let mut is_accounts = false;

            for attr in &item.attrs {
                if attr.path.is_ident("fankor") {
                    if let Ok(mut args) = attr.parse_args::<FnkMetaArgumentList>() {
                        args.error_on_duplicated()?;

                        is_accounts = args.pop_plain("accounts", true)?;

                        if args.pop_ident("account", true)?.is_some() {
                            return Err(Error::new(
                                input.span(),
                                "Account cannot be used with an enum",
                            ));
                        }

                        args.error_on_unknown()?;
                    } else {
                        return Err(Error::new(
                            attr.span(),
                            "The correct pattern is #[ts_gen(<meta_list>)]",
                        ));
                    };
                    break;
                }
            }

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
                            let field_name = case_converter.convert(field_name.to_string());
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
                            let field_name = case_converter.convert(field_name.to_string());
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
                            "export interface {} {{ type: '{}'; }}",
                            interface_name, variant_name_str,
                        ));

                        ts_schema_fields.push(format!(
                            "[{},'{}']",
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

                    // METHODS ----------------------------------------------------------------

                    serialize(buffer?: Buffer) {{
                        const writer = new fnk.FnkBorshWriter(buffer);
                        {}.serialize(writer, this);
                        return writer.toBuffer();
                    }}

                    equals(other: {}) {{
                        return this.data.type === other.data.type && fnk.equals((this.data as any)?.value, (other.data as any)?.value);
                    }}

                    clone(): {} {{
                        return new {}(fnk.clone(this.data));
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
                name_str,
                name_str,
                name_str,
                schema_constant_name,
                types_name,
                ts_interface_names.join("|"),
                ts_interfaces.join("\n")
            );

            let enum_schema_type = if is_accounts { "TAccountEnum" } else { "TEnum" };

            let ts_schema = format!(
                "export class {} implements fnk.FnkBorshSchema<{}> {{
                    innerSchema = null as any as ReturnType<{}['initSchema']>;

                    // METHODS ----------------------------------------------------------------

                    initSchema() {{
                        const innerSchema = fnk.{}([{}] as const);
                        this.innerSchema = innerSchema;
                        return innerSchema;
                    }}

                    serialize(writer: fnk.FnkBorshWriter, value: {}) {{
                        this.innerSchema.serialize(writer, value.data);
                    }}

                    deserialize(reader: fnk.FnkBorshReader) {{
                        const result = this.innerSchema.deserialize(reader);
                        return new {}(result as {});
                    }}
                }}",
                schema_name,
                name_str,
                schema_name,
                enum_schema_type,
                ts_schema_fields.join(","),
                name_str,
                name_str,
                types_name,
            );

            let ts_schema_use_method_name = format!("use{}", schema_name);
            let ts_schema_use_method_call = format!("{}()", ts_schema_use_method_name);
            let ts_schema_constant = format!(
                "export const {} = {};",
                schema_constant_name, ts_schema_use_method_call,
            );

            let ts_schema_use_method = format!(
                "const {} = (() => {{
                    let variable: {} | null = null;
                    return () => {{
                        if (variable === null) {{
                            variable = new {}();
                            variable.initSchema();
                        }}

                        return variable
                    }};
                }})();",
                ts_schema_use_method_name, schema_name, schema_name,
            );

            let result = quote! {
                #[automatically_derived]
                impl #impl_generics fankor::prelude::TsTypeGen for #name #ty_generics #where_clause {
                    fn value(&self) -> std::borrow::Cow<'static, str> {
                        unreachable!()
                    }

                    fn value_type() -> std::borrow::Cow<'static, str> {
                        std::borrow::Cow::Borrowed(#name_str)
                    }

                    fn schema_name() -> std::borrow::Cow<'static, str> {
                        std::borrow::Cow::Borrowed(#ts_schema_use_method_call)
                    }

                    fn generate_type(registered_types: &mut fankor::prelude::TsTypesCache) -> std::borrow::Cow<'static, str> {
                        use fankor::prelude::TsTypeGen;
                        let name = Self::value_type();

                        if registered_types.contains_key(&name) {
                            return name;
                        }

                        // Prevents infinite recursion.
                        registered_types.insert(name.clone(), std::borrow::Cow::Borrowed(""));

                        let ts_type = #ts_type.to_string() #(#type_replacements)*;
                        *registered_types.get_mut(&name).unwrap() = std::borrow::Cow::Owned(ts_type);

                        name
                    }

                    fn generate_schema(registered_schemas: &mut fankor::prelude::TsTypesCache) -> std::borrow::Cow<'static, str> {
                        use fankor::prelude::TsTypeGen;
                        let name = Self::schema_name();

                        if registered_schemas.contains_key(&name) {
                            return name;
                        }

                        // Prevents infinite recursion.
                        registered_schemas.insert(name.clone(), std::borrow::Cow::Borrowed(""));

                        let ts_schema = #ts_schema.to_string() #(#schema_replacements)*;
                        *registered_schemas.get_mut(&name).unwrap() = std::borrow::Cow::Owned(ts_schema);

                        name
                    }

                    fn generate_schema_constant(registered_constants: &mut fankor::prelude::TsTypesCache) {
                        use fankor::prelude::TsTypeGen;
                        let name = Self::schema_name();

                        if registered_constants.contains_key(&name) {
                            return;
                        }

                        let ts_schema = #ts_schema_constant .to_string();
                        registered_constants.insert(name.clone(), std::borrow::Cow::Owned(ts_schema));
                    }

                    fn generate_schema_use_method(registered_use_methods: &mut fankor::prelude::TsTypesCache) {
                        use fankor::prelude::TsTypeGen;
                        let name = Self::schema_name();

                        if registered_use_methods.contains_key(&name) {
                            return;
                        }

                        let ts_schema = #ts_schema_use_method .to_string();
                        registered_use_methods.insert(name.clone(), std::borrow::Cow::Owned(ts_schema));
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
        #[cfg(feature = "ts-gen")]
        #[automatically_derived]
        #[allow(non_snake_case)]
        mod #test_name {
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
