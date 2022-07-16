use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const INSTRUCTION_DISCRIMINATOR_SIZE: u8 = 8;

/// The configuration for the building process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorInstructionsConfig {
    /// The number of bytes of the instruction discriminators.
    /// Default: [INSTRUCTION_DISCRIMINATOR_SIZE](crate::fankor::config::INSTRUCTION_DISCRIMINATOR_SIZE)
    pub discriminator_size: Option<u8>,

    /// A manual list of instruction discriminators.
    #[serde(default)]
    pub discriminators: HashMap<String, Vec<u8>>,
}

impl FankorInstructionsConfig {
    // METHODS ----------------------------------------------------------------

    pub fn fill_with_defaults(&mut self) {
        // Validate the instruction discriminator size.
        match &self.discriminator_size {
            Some(v) => {
                if *v > 32 {
                    panic!("The instruction_discriminator_size cannot be greater than 32");
                }

                if *v < 1 {
                    panic!("The instruction_discriminator_size cannot be less than 1");
                }
            }
            None => {
                self.discriminator_size = Some(INSTRUCTION_DISCRIMINATOR_SIZE);
            }
        }

        // Note: the instruction discriminators will be validated when they are used.
    }

    pub fn get_discriminator(&self, name: &str) -> Option<Vec<u8>> {
        let result = match self.discriminators.get(name).cloned() {
            Some(v) => v,
            None => return None,
        };

        if result.len() != self.discriminator_size.unwrap() as usize {
            panic!(
                "The discriminator for instruction {} is not the correct size",
                name
            );
        }

        Some(result)
    }
}

impl Default for FankorInstructionsConfig {
    fn default() -> Self {
        FankorInstructionsConfig {
            discriminator_size: Some(INSTRUCTION_DISCRIMINATOR_SIZE),
            discriminators: HashMap::new(),
        }
    }
}
