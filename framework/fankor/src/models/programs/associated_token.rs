use crate::traits::ProgramType;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Copy, Clone)]
pub struct AssociatedToken;

impl ProgramType for AssociatedToken {
    fn name() -> &'static str {
        "AssociatedToken"
    }

    fn address() -> &'static Pubkey {
        &spl_associated_token_account::ID
    }
}
