use crate::ts_gen::types::{TsTypeGen, TsTypesCache};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// Contains the info for building the IDL.
pub struct DataContext {
    pub program_name: String,
    pub accounts: HashSet<Cow<'static, str>>,
    pub account_types: TsTypesCache,
    pub account_schemas: TsTypesCache,

    // Type-value pairs.
    pub constants: HashMap<&'static str, (Cow<'static, str>, Cow<'static, str>)>,
}

impl DataContext {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new IDL build context.
    pub fn new() -> DataContext {
        DataContext {
            program_name: "program".to_string(),
            accounts: HashSet::new(),
            account_types: TsTypesCache::new(),
            account_schemas: TsTypesCache::new(),
            constants: HashMap::new(),
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Adds an account.
    pub fn add_account<T: TsTypeGen>(&mut self) -> Result<(), String> {
        let name = T::value_type();

        if self.accounts.contains(&name) {
            return Err(format!("Duplicated account name: '{}'", name));
        }

        T::generate_type(&mut self.account_types);
        T::generate_schema(&mut self.account_schemas);

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

    /// Builds the TypeScript file from the data stored in the context.
    pub fn build_ts_file(&mut self) -> String {
        let mut buffer = String::new();

        self.build_constants(&mut buffer);
        self.build_types_and_schemas(&mut buffer);

        buffer
    }

    /// Builds constants part.
    pub fn build_constants(&mut self, writer: &mut String) {
        for (name, (ty, value)) in self.constants.iter() {
            writer.push_str(format!("export const {}: {} = {};\n", name, ty, value).as_str());
        }
    }

    /// Builds types and schemas part.
    pub fn build_types_and_schemas(&mut self, writer: &mut String) {
        for (_name, type_definition) in self.account_types.iter() {
            writer.push_str(&type_definition);
        }

        for (_name, schema) in self.account_schemas.iter() {
            writer.push_str(&schema);
        }
    }
}
