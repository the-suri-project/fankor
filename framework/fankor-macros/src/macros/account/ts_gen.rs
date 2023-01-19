use crate::Result;
use convert_case::{Case, Converter};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Item;

pub fn ts_gen(input: &Item) -> Result<TokenStream> {
    let case_converter = Converter::new().from_case(Case::Snake).to_case(Case::Camel);

    // Process input.
    let item = match &input {
        Item::Struct(item) => item,
        _ => unreachable!(),
    };

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
    let mut equals_method_conditions = Vec::new();

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

        equals_method_conditions.push(format!(
            "fnk.equals(this.{}, other.{})",
            field_name_str, field_name_str
        ));
    }

    let ts_type = format!(
        "export class {} {{
            // CONSTRUCTORS -----------------------------------------------------------

            constructor({}) {{}}

            // GETTERS ----------------------------------------------------------------

            static get discriminant() {{
                return _r_discriminant_r_;
            }}

            static get schema() {{
                return {};
            }}

            // METHODS ----------------------------------------------------------------

            serialize(buffer?: Buffer) {{
                const writer = new fnk.FnkBorshWriter(buffer);
                {}.serialize(writer, this);
                return writer.toByteArray();
            }}

            equals(other: {}) {{
                return {};
            }}

            // STATIC METHODS ---------------------------------------------------------

            static async getOnChainAccountByAddress(
                connection: solana.Connection,
                address: solana.PublicKey
            ): Promise<fnk.AccountResult<{}> | null> {{
                let account = await connection.getAccountInfo(
                    address,
                    connection.commitment
                );

                if (account) {{
                    if (account.owner == ID) {{
                        let buf = account.data;
                        let data = this.deserialize(buf);
                        return {{
                            address,
                            account,
                            data,
                        }};
                    }}
                }}

                return null;
            }}

            static deserialize(buffer: Buffer, offset?: number) {{
                const reader = new fnk.FnkBorshReader(buffer, offset);
                return {}.deserialize(reader);
            }}
        }}",
        name_str,
        ts_constructor_params.join(","),
        schema_constant_name,
        schema_constant_name,
        name_str,
        equals_method_conditions.join(" && "),
        name_str,
        schema_constant_name,
    );

    let ts_schema = format!(
        "export class {} implements fnk.FnkBorshSchema<{}> {{
            innerSchema = fnk.TStruct([
                ['discriminant', fnk.U8],
                {}
            ] as const);

            // METHODS ----------------------------------------------------------------

            serialize(writer: fnk.FnkBorshWriter, value: {}) {{
                this.innerSchema.serialize(writer, {{
                    discriminant: _r_discriminant_r_,
                    ...value
                }});
            }}

            deserialize(reader: fnk.FnkBorshReader) {{
                const result = this.innerSchema.deserialize(reader);
                return new {}({});
            }}
        }}",
        schema_name,
        name_str,
        ts_schema_fields.join(","),
        name_str,
        name_str,
        ts_schema_constructor_args.join(","),
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
                }}

                return variable
            }};
        }})();",
        ts_schema_use_method_name, schema_name, schema_name,
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
                std::borrow::Cow::Borrowed(#ts_schema_use_method_call)
            }

            fn generate_type(registered_types: &mut ::fankor::prelude::ts_gen::types::TsTypesCache) -> std::borrow::Cow<'static, str> {
                use ::fankor::prelude::ts_gen::types::TsTypeGen;
                let name = Self::value_type();

                if registered_types.contains_key(&name) {
                    return name;
                }

                // Prevents infinite recursion.
                registered_types.insert(name.clone(), std::borrow::Cow::Borrowed(""));

                let ts_type = #ts_type #(#constructor_replacements)* .replace("_r_discriminant_r_", &Self::discriminant().to_string());
                *registered_types.get_mut(&name).unwrap() = std::borrow::Cow::Owned(ts_type);

                name
            }

            fn generate_schema(registered_schemas: &mut ::fankor::prelude::ts_gen::types::TsTypesCache) -> std::borrow::Cow<'static, str> {
                use ::fankor::prelude::ts_gen::types::TsTypeGen;
                let name = Self::schema_name();

                if registered_schemas.contains_key(&name) {
                    return name;
                }

                // Prevents infinite recursion.
                registered_schemas.insert(name.clone(), std::borrow::Cow::Borrowed(""));

                let ts_schema = #ts_schema #(#schema_replacements)* .replace("_r_discriminant_r_", &Self::discriminant().to_string());
                *registered_schemas.get_mut(&name).unwrap() = std::borrow::Cow::Owned(ts_schema);

                name
            }

            fn generate_schema_constant(registered_constants: &mut ::fankor::prelude::ts_gen::types::TsTypesCache) {
                use ::fankor::prelude::ts_gen::types::TsTypeGen;
                let name = Self::schema_name();

                if registered_constants.contains_key(&name) {
                    return;
                }

                let ts_schema = #ts_schema_constant .to_string();
                registered_constants.insert(name.clone(), std::borrow::Cow::Owned(ts_schema));
            }

            fn generate_schema_use_method(registered_use_methods: &mut ::fankor::prelude::ts_gen::types::TsTypesCache) {
                use ::fankor::prelude::ts_gen::types::TsTypeGen;
                let name = Self::schema_name();

                if registered_use_methods.contains_key(&name) {
                    return;
                }

                let ts_schema = #ts_schema_use_method .to_string();
                registered_use_methods.insert(name.clone(), std::borrow::Cow::Owned(ts_schema));
            }
        }
    };

    let test_name = format_ident!("__ts_gen_test__account_{}", name_str);
    let test_name_str = test_name.to_string();
    let result = quote! {
        #[cfg(feature = "ts-gen")]
        #[automatically_derived]
        #[allow(non_snake_case)]
        pub mod #test_name {
            use super::*;

            #result

            #[test]
            fn build() {
                 // Register action.
                crate::__ts_gen_test__setup::BUILD_CONTEXT.register_action(#test_name_str, file!(), move |action_context| {
                    action_context.add_account::<#name>().unwrap();
                })
            }
        }
    };

    Ok(result.into())
}
