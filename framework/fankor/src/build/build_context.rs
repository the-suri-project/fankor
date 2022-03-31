use crate::build::types::IdlType;
use crate::build::FankorConfig;
use std::collections::HashMap;

/// Contains the info for building the IDL.
pub struct IdlBuildContext {
    constants: HashMap<String, IdlConstant>,
    error_file: Option<String>,
}

impl IdlBuildContext {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new IDL build context.
    pub fn new() -> IdlBuildContext {
        IdlBuildContext {
            constants: HashMap::new(),
            error_file: None,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the error file.
    pub fn error_file(&self) -> Option<&String> {
        self.error_file.as_ref()
    }

    // SETTERS ----------------------------------------------------------------

    /// Sets the error file.
    pub fn set_error_file(&mut self, error_file: String) {
        self.error_file = Some(error_file);
    }

    // METHODS ----------------------------------------------------------------

    /// Adds a constant to the IDL build context.
    pub fn add_constant(
        &mut self,
        name: String,
        kind: IdlType,
        value: serde_json::Value,
    ) -> Result<(), String> {
        if self.constants.contains_key(&name) {
            return Err(format!("Duplicated constant name: '{}'", name));
        }

        self.constants.insert(name, IdlConstant { kind, value });

        Ok(())
    }

    /// Builds the IDL from the data stored in the context.
    pub fn build_idl(&mut self, config: &FankorConfig) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "name".to_string(),
            serde_json::Value::String(config.program_name.clone()),
        );
        map.insert("constants".to_string(), self.build_idl_constants());

        serde_json::Value::Object(map)
    }

    /// Builds the IDL constants part.
    pub fn build_idl_constants(&mut self) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        for (name, constant) in self.constants.drain() {
            let mut constant_map = serde_json::Map::new();
            constant_map.insert("type".to_string(), constant.kind.to_idl_value());
            constant_map.insert("value".to_string(), constant.value.clone());

            map.insert(name, serde_json::Value::Object(constant_map));
        }

        serde_json::Value::Object(map)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct IdlConstant {
    /// The type of the constant.
    kind: IdlType,

    /// The value of the constant.
    value: serde_json::Value,
}
