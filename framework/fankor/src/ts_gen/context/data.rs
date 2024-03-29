use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use convert_case::{Case, Converter};

use crate::traits::{TsInstructionGen, TsTypeGen, TsTypesCache};

/// Contains the info for building the IDL.
pub struct DataContext {
    pub program_name: &'static str,
    pub accounts: HashSet<Cow<'static, str>>,
    pub account_types: TsTypesCache,
    pub account_type_extensions: TsTypesCache,
    pub account_schemas: TsTypesCache,
    pub account_schemas_use_methods: TsTypesCache,
    pub account_schemas_constants: TsTypesCache,
    pub get_meta_methods: TsTypesCache,
    pub program_methods: TsTypesCache,

    // Type-value pairs.
    pub constants: HashMap<&'static str, (Cow<'static, str>, Cow<'static, str>)>,
}

impl DataContext {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new IDL build context.
    pub fn new() -> DataContext {
        DataContext {
            program_name: "program",
            accounts: HashSet::new(),
            account_types: TsTypesCache::new(),
            account_type_extensions: HashMap::new(),
            account_schemas: TsTypesCache::new(),
            account_schemas_use_methods: TsTypesCache::new(),
            account_schemas_constants: TsTypesCache::new(),
            get_meta_methods: HashMap::new(),
            program_methods: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Adds an account.
    pub fn set_context_name(&mut self, name: &'static str) -> Result<(), String> {
        self.program_name = name;

        Ok(())
    }

    /// Adds an account.
    pub fn add_created_type(
        &mut self,
        name: &'static str,
        data: Cow<'static, str>,
    ) -> Result<(), String> {
        if self.accounts.contains(name) {
            return Err(format!("Duplicated account name: '{}'", name));
        }

        self.account_types.insert(Cow::Borrowed(name), data);

        Ok(())
    }

    /// Adds an account.
    pub fn add_account<T: TsTypeGen>(&mut self) -> Result<(), String> {
        let name = T::value_type();

        if self.accounts.contains(&name) {
            return Err(format!("Duplicated account name: '{}'", name));
        }

        T::generate_type(&mut self.account_types);
        T::generate_schema(&mut self.account_schemas);
        T::generate_schema_constant(&mut self.account_schemas_constants);
        T::generate_schema_use_method(&mut self.account_schemas_use_methods);

        Ok(())
    }

    /// Adds a constant.
    pub fn add_constant<T: TsTypeGen>(
        &mut self,
        name: &'static str,
        value: T,
    ) -> Result<(), String> {
        if self.constants.contains_key(&name) {
            return Err(format!("Duplicated constant name: '{}'", name));
        }

        self.constants
            .insert(name, (T::value_type(), value.value()));

        Ok(())
    }

    /// Adds an instruction account.
    pub fn add_instruction_account<T: TsInstructionGen>(&mut self) -> Result<(), String> {
        let name = T::value_type();

        if self.accounts.contains(&name) {
            return Err(format!("Duplicated instruction account: '{}'", name));
        }

        T::generate_type(&mut self.account_types);

        let get_metas_method = format!(
            "function getMetasOf{}(accounts: {}, accountMetas: solana.AccountMeta[], writer: fnk.FnkBorshWriter) {{
                {}
            }}",
            name,
            name,
            T::get_account_metas(Cow::Borrowed("accounts"), false, false),
        );

        self.get_meta_methods
            .insert(name, Cow::Owned(get_metas_method));

        Ok(())
    }

    /// Adds a program method.
    pub fn add_program_method<T: TsInstructionGen>(
        &mut self,
        discriminant_name: &'static str,
        variant_name: &'static str,
    ) -> Result<(), String> {
        let case_converter = Converter::new()
            .from_case(Case::Pascal)
            .to_case(Case::Camel);
        let name = Cow::Owned(case_converter.convert(variant_name));

        if self.program_methods.contains_key(&name) {
            return Err(format!("Duplicated program method: '{}'", name));
        }

        let accounts_type = T::value_type();
        let method = format!(
            "{}(accounts: {}) {{
                const writer = new fnk.FnkBorshWriter();
                writer.writeByte({}.{});
                const accountMetas: solana.AccountMeta[] = [];

                getMetasOf{}(accounts, accountMetas, writer);

                return new solana.TransactionInstruction({{
                    keys: accountMetas,
                    programId: ID,
                    data: writer.toBuffer()
                }});
            }}",
            name, accounts_type, discriminant_name, variant_name, accounts_type,
        );

        self.program_methods.insert(name, Cow::Owned(method));

        Ok(())
    }

    /// Adds an account type extension.
    pub fn add_account_type_extensions(
        &mut self,
        name: &'static str,
        data: Cow<'static, str>,
    ) -> Result<(), String> {
        if self.program_methods.contains_key(&Cow::Borrowed(name)) {
            return Err(format!("Duplicated account type extension: '{}'", name));
        }

        self.account_type_extensions
            .insert(Cow::Borrowed(name), data);

        Ok(())
    }

    /// Builds the TypeScript file from the data stored in the context.
    pub fn build_ts_file(&mut self) -> String {
        let mut buffer = String::new();

        // Imports.
        buffer.push_str("import * as solana from '@solana/web3.js';");
        buffer.push_str("import * as fnk from '@suri-project/fankor/dist/esm';");
        buffer.push_str("import BN from 'bn.js';");

        // Build constants part.
        let mut constants = self.constants.iter().collect::<Vec<_>>();
        constants.sort_by(|a, b| a.0.cmp(b.0));

        for (name, (ty, value)) in constants {
            buffer.push_str(format!("export const {}: {} = {};\n", name, ty, value).as_str());
        }

        // Build types.
        let mut account_types = self.account_types.iter().collect::<Vec<_>>();
        account_types.sort_by(|a, b| a.0.cmp(b.0));

        for (_name, type_definition) in account_types {
            buffer.push_str(type_definition);
        }

        // Build types extensions.
        let mut account_type_extensions = self.account_type_extensions.iter().collect::<Vec<_>>();
        account_type_extensions.sort_by(|a, b| a.0.cmp(b.0));

        for (_name, type_definition) in account_type_extensions {
            buffer.push_str(type_definition);
        }

        // Build schemas.
        let mut account_schemas = self.account_schemas.iter().collect::<Vec<_>>();
        account_schemas.sort_by(|a, b| a.0.cmp(b.0));

        for (_name, schema) in account_schemas {
            buffer.push_str(schema);
        }

        // Build schema use methods.
        let mut account_schemas_use_methods =
            self.account_schemas_use_methods.iter().collect::<Vec<_>>();
        account_schemas_use_methods.sort_by(|a, b| a.0.cmp(b.0));

        for (_name, use_method) in account_schemas_use_methods {
            buffer.push_str(use_method);
        }

        // Build schema constants.
        let mut account_schemas_constants =
            self.account_schemas_constants.iter().collect::<Vec<_>>();
        account_schemas_constants.sort_by(|a, b| a.0.cmp(b.0));

        for (_name, constant) in account_schemas_constants {
            buffer.push_str(constant);
        }

        // Build get meta methods.
        let mut get_meta_methods = self.get_meta_methods.iter().collect::<Vec<_>>();
        get_meta_methods.sort_by(|a, b| a.0.cmp(b.0));

        for (_name, method) in get_meta_methods {
            buffer.push_str(method);
        }

        // Build program methods.
        let mut program_methods = self.program_methods.iter().collect::<Vec<_>>();
        program_methods.sort_by(|a, b| a.0.cmp(b.0));

        buffer.push_str("export const instructions = {");
        for (_name, method) in program_methods {
            buffer.push_str(method);
            buffer.push(',');
        }
        buffer.push_str("};");

        buffer
    }
}

impl Default for DataContext {
    fn default() -> Self {
        Self::new()
    }
}
