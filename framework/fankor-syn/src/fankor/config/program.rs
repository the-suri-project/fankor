use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::str::FromStr;

/// The configuration for the building process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorProgramConfig {
    /// The pubkey of the program.
    pub pubkey: String,
}

impl FankorProgramConfig {
    // METHODS ----------------------------------------------------------------

    pub fn validate(&self) {
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
            pubkey: Pubkey::default().to_string(),
        }
    }
}
