pub use accounts::*;
pub use instructions::*;
pub use program::*;

mod accounts;
mod instructions;
mod program;

use serde::{Deserialize, Serialize};

/// The Fankor configuration.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FankorConfig {
    /// The program configuration.
    pub program: FankorProgramConfig,

    #[serde(default)]
    pub accounts: FankorAccountsConfig,

    #[serde(default)]
    pub instructions: FankorInstructionsConfig,
}

impl FankorConfig {
    // METHODS ----------------------------------------------------------------

    pub fn validate(&self) {
        self.program.validate();
        self.accounts.validate();
        self.instructions.validate();
    }
}
