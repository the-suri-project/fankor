use crate::models::Token;
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

impl AssociatedToken {
    // METHODS ----------------------------------------------------------------

    pub fn get_pda_seeds<'a>(
        wallet_address: &'a Pubkey,
        token_mint_address: &'a Pubkey,
    ) -> [&'a [u8]; 3] {
        [
            wallet_address.as_ref(),
            Token::address().as_ref(),
            token_mint_address.as_ref(),
        ]
    }
}
