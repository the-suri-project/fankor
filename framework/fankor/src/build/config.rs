use serde::{Deserialize, Serialize};

pub const INITIAL_DELAY_CONFIG: u64 = 1000;

/// The configuration for the building process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorConfig {
    /// The name of the program.
    pub program_name: String,

    /// The initial delay in milliseconds.
    pub initial_delay: Option<u64>,
}

impl FankorConfig {
    // METHODS ----------------------------------------------------------------

    pub fn fill_with_defaults(&mut self) {
        if self.initial_delay.is_none() {
            self.initial_delay = Some(INITIAL_DELAY_CONFIG);
        }
    }

    pub fn validate(&self) {
        for char in self.program_name.chars() {
            if !char.is_ascii_alphanumeric() && char != '_' {
                panic!("The program name must be alphanumeric or underscore.");
            }
        }
    }
}

impl Default for FankorConfig {
    fn default() -> Self {
        FankorConfig {
            program_name: "smart_contract".to_string(),
            initial_delay: Some(INITIAL_DELAY_CONFIG),
        }
    }
}
