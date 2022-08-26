use crate::traits::Program;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Copy, Clone)]
pub struct AssociatedToken;

impl Program for AssociatedToken {
    fn name() -> &'static str {
        "AssociatedToken"
    }

    fn address() -> &'static Pubkey {
        &spl_associated_token_account::ID
    }
}
