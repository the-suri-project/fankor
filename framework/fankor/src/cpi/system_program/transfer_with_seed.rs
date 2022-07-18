use crate::errors::Error::ProgramError;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub struct CpiTransferWithSeed<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub base: AccountInfo<'info>,
}

pub fn transfer_with_seed(
    accounts: CpiTransferWithSeed,
    from_seed: String,
    from_owner: &Pubkey,
    lamports: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::transfer_with_seed(
        accounts.from.key,
        accounts.base.key,
        from_seed,
        from_owner,
        accounts.to.key,
        lamports,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.from, accounts.base, accounts.to],
        signer_seeds,
    )
    .map_or_else(|e| Err(ProgramError(e)), |_| Ok(()))
}
