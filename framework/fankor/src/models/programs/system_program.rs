use solana_program::pubkey::Pubkey;

use crate::traits::ProgramType;

#[derive(Debug, Copy, Clone)]
pub struct System;

impl ProgramType for System {
    fn name() -> &'static str {
        "System"
    }

    fn address() -> &'static Pubkey {
        &solana_program::system_program::ID
    }
}
