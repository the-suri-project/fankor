pub use instructions::*;
pub use program::*;

mod instructions;
mod program;

use serde::{Deserialize, Serialize};

/// The Fankor configuration.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FankorConfig {
    /// The program configuration.
    pub program: FankorProgramConfig,

    #[serde(default)]
    pub instructions: FankorInstructionsConfig,
}

impl FankorConfig {
    // METHODS ----------------------------------------------------------------

    pub fn validate(&self) {
        self.program.validate();
        self.instructions.validate();
    }
}
