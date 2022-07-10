use serde::{Deserialize, Serialize};

pub const TASK_WAIT_INTERVAL_DEFAULT: u64 = 100;
pub const TASK_WAIT_TIMEOUT_DEFAULT: u64 = 2000;

/// The configuration for the building process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FankorBuildConfig {
    /// The initial delay in milliseconds.
    pub task_wait_interval: Option<u64>,

    /// The maximum milliseconds to wait for all tasks to get registered.
    pub task_wait_timeout: Option<u64>,
}

impl FankorBuildConfig {
    // METHODS ----------------------------------------------------------------

    pub fn fill_with_defaults(&mut self) {
        if self.task_wait_interval.is_none() {
            self.task_wait_interval = Some(TASK_WAIT_INTERVAL_DEFAULT);
        }

        if self.task_wait_timeout.is_none() {
            self.task_wait_timeout = Some(TASK_WAIT_TIMEOUT_DEFAULT);
        }
    }
}

impl Default for FankorBuildConfig {
    fn default() -> Self {
        FankorBuildConfig {
            task_wait_interval: Some(TASK_WAIT_INTERVAL_DEFAULT),
            task_wait_timeout: Some(TASK_WAIT_TIMEOUT_DEFAULT),
        }
    }
}
