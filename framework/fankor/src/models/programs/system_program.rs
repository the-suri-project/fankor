use crate::traits::Program;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Copy, Clone)]
pub struct System;

impl Program for System {
    fn address() -> &'static Pubkey {
        &solana_program::system_program::ID
    }
}
