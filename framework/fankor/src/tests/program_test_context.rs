//! Code based on https://github.com/halbornteam/solana-test-framework

use async_trait::async_trait;
use solana_program_test::{ProgramTestContext, ProgramTestError};
use solana_sdk::sysvar::clock::Clock;

#[async_trait]
pub trait ProgramTestContextExtension {
    /// Calculate slot number from the provided timestamp
    async fn warp_to_timestamp(&mut self, timestamp: i64) -> Result<(), ProgramTestError>;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[async_trait]
impl ProgramTestContextExtension for ProgramTestContext {
    async fn warp_to_timestamp(&mut self, timestamp: i64) -> Result<(), ProgramTestError> {
        const NANOSECONDS_IN_SECOND: u128 = 1_000_000_000;

        let mut clock: Clock = self.banks_client.get_sysvar().await.unwrap();
        let now = clock.unix_timestamp;
        let current_slot = clock.slot;
        clock.unix_timestamp = timestamp;

        if now >= timestamp {
            return Err(ProgramTestError::InvalidWarpSlot);
        }

        let ns_per_slot = self.genesis_config().ns_per_slot();
        let timestamp_diff_s = (timestamp / now) as u128;
        let timestamp_diff_ns = timestamp_diff_s * NANOSECONDS_IN_SECOND;

        let slots = (timestamp_diff_ns / ns_per_slot) as u64;

        self.set_sysvar(&clock);
        self.warp_to_slot(current_slot + slots)?;

        Ok(())
    }
}
