pub use build::*;
pub use program::*;
mod build;
mod program;

use serde::{Deserialize, Serialize};

/// The Fankor configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorConfig {
    /// The program configuration.
    pub program: FankorProgramConfig,

    /// The initial delay in milliseconds.
    pub build: Option<FankorBuildConfig>,
}

impl FankorConfig {
    // GETTERS ----------------------------------------------------------------

    /// Returns the build configuration.
    pub fn build(&self) -> &FankorBuildConfig {
        self.build.as_ref().unwrap()
    }

    // METHODS ----------------------------------------------------------------

    pub fn fill_with_defaults(&mut self) {
        if let Some(build) = &mut self.build {
            build.fill_with_defaults();
        } else {
            self.build = Some(FankorBuildConfig::default());
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
            build: Some(FankorBuildConfig::default()),
        }
    }
}
