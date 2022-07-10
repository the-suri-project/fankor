use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::str::FromStr;

/// The configuration for the building process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorProgramConfig {
    /// The name of the program.
    pub name: String,

    /// The pubkey of the program.
    pub pubkey: String,
}

impl FankorProgramConfig {
    // METHODS ----------------------------------------------------------------

    pub fn validate(&self) {
        for char in self.name.chars() {
            if !char.is_ascii_alphanumeric() && char != '_' {
                panic!("The program name must be alphanumeric or underscore.");
            }
        }

        let pubkey = Pubkey::from_str(self.pubkey.as_str())
            .expect("The program pubkey is not a valid pubkey.");

        if !pubkey.is_on_curve() {
            panic!("The program pubkey is invalid because it is not on the curve.");
        }
    }
}

impl Default for FankorProgramConfig {
    fn default() -> Self {
        FankorProgramConfig {
            name: "SmartContract".to_string(),
            pubkey: Pubkey::default().to_string(),
        }
    }
}
