use serde::{Deserialize, Serialize};

pub const INITIAL_DELAY_CONFIG: u64 = 1000;

/// The configuration for the building process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorBuildConfig {
    /// The initial delay in milliseconds.
    pub initial_delay: Option<u64>,
}

impl FankorBuildConfig {
    // METHODS ----------------------------------------------------------------

    pub fn fill_with_defaults(&mut self) {
        if self.initial_delay.is_none() {
            self.initial_delay = Some(INITIAL_DELAY_CONFIG);
        }
    }
}

impl Default for FankorBuildConfig {
    fn default() -> Self {
        FankorBuildConfig {
            initial_delay: Some(INITIAL_DELAY_CONFIG),
        }
    }
}
