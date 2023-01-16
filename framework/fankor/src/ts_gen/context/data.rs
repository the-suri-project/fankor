use crate::ts_gen::types::TsTypeGen;
use std::borrow::Cow;
use std::collections::HashMap;

/// Contains the info for building the IDL.
pub struct DataContext {
    pub program_name: String,
    pub constants: HashMap<&'static str, Cow<'static, str>>,
}

impl DataContext {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new IDL build context.
    pub fn new() -> DataContext {
        DataContext {
            program_name: "program".to_string(),
            constants: HashMap::new(),
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Adds a constant.
    pub fn add_constant<T: TsTypeGen>(
        &mut self,
        name: &'static str,
        value: T,
    ) -> Result<(), String> {
        if self.constants.contains_key(&name) {
            return Err(format!("Duplicated constant name: '{}'", name));
        }

        self.constants.insert(name, value.value());

        Ok(())
    }

    /// Builds the TypeScript file from the data stored in the context.
    pub fn build_ts_file(&mut self) -> String {
        let mut buffer = String::new();

        self.build_constants(&mut buffer);

        buffer
    }

    /// Builds constants part.
    pub fn build_constants(&mut self, writer: &mut String) {
        for (name, value) in self.constants.iter() {
            writer.push_str(format!("type {}_TYPE = {};\n", name, value).as_str());
            writer
                .push_str(format!("export const {}: {}_TYPE = {};\n", name, name, value).as_str());
        }
    }
}
