use crate::traits::ProgramType;
use solana_program::pubkey::Pubkey;

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
