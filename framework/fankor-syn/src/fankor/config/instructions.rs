use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The configuration for the building process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorInstructionsConfig {
    /// The number of bytes used to defina the instruction discriminators.
    /// Default: 1
    pub discriminator_size: u8,

    /// A manual list of instruction discriminators.
    #[serde(default)]
    pub discriminators: HashMap<String, Vec<u8>>,
}

impl FankorInstructionsConfig {
    // METHODS ----------------------------------------------------------------

    pub fn validate(&self) {
        // Validate the instruction discriminator size.
        if self.discriminator_size > 32 {
            panic!("The instruction_discriminator_size cannot be greater than 32");
        }

        if self.discriminator_size < 1 {
            panic!("The instruction_discriminator_size cannot be less than 1");
        }

        // Note: the instruction discriminators will be validated when they are used.
    }

    pub fn get_discriminator(&self, name: &str) -> Vec<u8> {
        let result = match self.discriminators.get(name).cloned() {
            Some(v) => v,
            None => panic!(
                "The discriminator for the instruction '{}' is missing",
                name
            ),
        };

        if result.len() != self.discriminator_size as usize {
            panic!(
                "The discriminator size of the instruction '{}' is incorrect",
                name
            );
        }

        result
    }
}

impl Default for FankorInstructionsConfig {
    fn default() -> Self {
        FankorInstructionsConfig {
            discriminator_size: 1,
            discriminators: HashMap::new(),
        }
    }
}
