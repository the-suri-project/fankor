use crate::build::types::{IdlType, IdlTypeMappable};
use fankor_syn::fankor::FankorConfig;
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
    pub fn add_constant<T: IdlTypeMappable>(
        &mut self,
        name: String,
        constant: &T,
    ) -> Result<(), String> {
        let kind = T::idl_type();
        let value = constant.map_value_to_json();

        if self.constants.contains_key(&name) {
            return Err(format!("Duplicated constant name: '{}'", name));
        }

        self.constants.insert(name, IdlConstant { kind, value });

        Ok(())
    }

    /// Builds the IDL from the data stored in the context.
    pub fn build_idl(&self, config: &FankorConfig) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "name".to_string(),
            serde_json::Value::String(config.program.name.clone()),
        );
        map.insert(
            "programId".to_string(),
            serde_json::Value::String(config.program.pubkey.clone()),
        );
        map.insert("constants".to_string(), self.build_idl_constants());

        serde_json::Value::Object(map)
    }

    /// Builds the IDL constants part.
    pub fn build_idl_constants(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        for (name, constant) in &self.constants {
            let mut constant_map = serde_json::Map::new();
            constant_map.insert("type".to_string(), constant.kind.map_type_to_json());
            constant_map.insert("value".to_string(), constant.value.clone());

            map.insert(name.clone(), serde_json::Value::Object(constant_map));
        }

        serde_json::Value::Object(map)
    }
}

impl Default for IdlBuildContext {
    fn default() -> Self {
        Self::new()
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
