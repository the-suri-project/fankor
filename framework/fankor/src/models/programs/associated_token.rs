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

    #[cfg(feature = "token-program")]
    pub fn get_pda_seeds<'a>(
        wallet_address: &'a Pubkey,
        token_mint_address: &'a Pubkey,
    ) -> [&'a [u8]; 3] {
        Self::get_pda_seeds_with_program(
            wallet_address,
            token_mint_address,
            crate::models::Token::address(),
        )
    }

    #[cfg(feature = "token-program-2022")]
    pub fn get_pda_seeds_2022<'a>(
        wallet_address: &'a Pubkey,
        token_mint_address: &'a Pubkey,
    ) -> [&'a [u8]; 3] {
        Self::get_pda_seeds_with_program(
            wallet_address,
            token_mint_address,
            crate::models::Token2022::address(),
        )
    }

    pub fn get_pda_seeds_with_program<'a>(
        wallet_address: &'a Pubkey,
        token_mint_address: &'a Pubkey,
        program_id: &'a Pubkey,
    ) -> [&'a [u8]; 3] {
        [
            wallet_address.as_ref(),
            program_id.as_ref(),
            token_mint_address.as_ref(),
        ]
    }
}
