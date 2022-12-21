use crate::errors::Error;
use crate::models::{Program, Token2022};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiBurn<'info> {
    pub from: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}

pub fn burn(
    program: &Program<Token2022>,
    accounts: CpiBurn,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token_2022::instruction::burn(
        program.address(),
        accounts.from.key,
        accounts.mint.key,
        accounts.authority.key,
        &[],
        amount,
    )?;

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.from, accounts.mint, accounts.authority],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiBurnMultisig<'info> {
    pub from: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn burn_multisig(
    program: &Program<Token2022>,
    accounts: CpiBurnMultisig,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token_2022::instruction::burn(
        program.address(),
        accounts.from.key,
        accounts.mint.key,
        accounts.authority.key,
        &signer_pubkeys,
        amount,
    )?;

    let mut infos = Vec::with_capacity(3 + accounts.signers.len());
    infos.push(accounts.from);
    infos.push(accounts.mint);
    infos.push(accounts.authority);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
