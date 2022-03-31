use serde::{Deserialize, Serialize};

/// The configuration for the building process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorConfig {
    /// The initial delay in milliseconds.
    pub initial_delay: u64,
}

impl Default for FankorConfig {
    fn default() -> Self {
        FankorConfig {
            initial_delay: 1000,
        }
    }
}
