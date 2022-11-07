use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The configuration for the building process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorAccountsConfig {
    /// The number of bytes used to defina the account discriminators.
    /// Default: 1
    pub discriminator_size: u8,

    /// A manual list of account discriminators.
    #[serde(default)]
    pub discriminators: HashMap<String, Vec<u8>>,
}

impl FankorAccountsConfig {
    // METHODS ----------------------------------------------------------------

    pub fn validate(&self) {
        // Validate the account discriminator size.
        if self.discriminator_size > 32 {
            panic!("The account_discriminator_size cannot be greater than 32");
        }

        if self.discriminator_size < 1 {
            panic!("The account_discriminator_size cannot be less than 1");
        }

        // Note: the account discriminators will be validated if used.
    }

    pub fn get_discriminator(&self, name: &str) -> Vec<u8> {
        let result = match self.discriminators.get(name).cloned() {
            Some(v) => v,
            None => panic!("The discriminator for the account '{}' is missing", name),
        };

        if result.len() != self.discriminator_size as usize {
            panic!(
                "The discriminator size of the account '{}' is incorrect",
                name
            );
        }

        result
    }
}

impl Default for FankorAccountsConfig {
    fn default() -> Self {
        FankorAccountsConfig {
            discriminator_size: 1,
            discriminators: HashMap::new(),
        }
    }
}
