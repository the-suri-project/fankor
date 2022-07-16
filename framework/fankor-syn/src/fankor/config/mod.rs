pub use accounts::*;
pub use instructions::*;
pub use program::*;

mod accounts;
mod instructions;
mod program;

use serde::{Deserialize, Serialize};

/// The Fankor configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorConfig {
    /// The program configuration.
    pub program: FankorProgramConfig,

    #[serde(default)]
    pub accounts: Option<FankorAccountsConfig>,

    #[serde(default)]
    pub instructions: Option<FankorInstructionsConfig>,
}

impl FankorConfig {
    // GETTERS ----------------------------------------------------------------

    /// Returns the build configuration.
    pub fn build(&self) -> &FankorAccountsConfig {
        self.accounts.as_ref().unwrap()
    }

    // METHODS ----------------------------------------------------------------

    pub fn fill_with_defaults(&mut self) {
        if let Some(build) = &mut self.accounts {
            build.fill_with_defaults();
        } else {
            self.accounts = Some(FankorAccountsConfig::default());
        }

        if let Some(build) = &mut self.instructions {
            build.fill_with_defaults();
        } else {
            self.instructions = Some(FankorInstructionsConfig::default());
        }
    }

    pub fn validate(&self) {
        self.program.validate();
    }
}

impl Default for FankorConfig {
    fn default() -> Self {
        FankorConfig {
            program: FankorProgramConfig::default(),
            accounts: Some(FankorAccountsConfig::default()),
            instructions: Some(FankorInstructionsConfig::default()),
        }
    }
}
