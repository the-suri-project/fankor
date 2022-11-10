pub use program::*;

mod program;

use serde::{Deserialize, Serialize};

/// The Fankor configuration.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FankorConfig {
    /// The program configuration.
    pub program: FankorProgramConfig,
}

impl FankorConfig {
    // METHODS ----------------------------------------------------------------

    pub fn validate(&self) {
        self.program.validate();
    }
}
