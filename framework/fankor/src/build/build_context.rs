use crate::build::types::IdlType;
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
