use crate::errors::Error;
use crate::models::{Program, Token2022};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiMintTo<'info> {
    pub mint: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}

pub fn mint_to(
    program: &Program<Token2022>,
    accounts: CpiMintTo,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token_2022::instruction::mint_to(
        program.address(),
        accounts.mint.key,
        accounts.to.key,
        accounts.authority.key,
        &[],
        amount,
    )?;

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.mint, accounts.to, accounts.authority],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiMintToMultisig<'info> {
    pub mint: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn mint_to_multisig(
    program: &Program<Token2022>,
    accounts: CpiMintToMultisig,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token_2022::instruction::mint_to(
        program.address(),
        accounts.mint.key,
        accounts.to.key,
        accounts.authority.key,
        &signer_pubkeys,
        amount,
    )?;

    let mut infos = Vec::with_capacity(3 + accounts.signers.len());
    infos.push(accounts.mint);
    infos.push(accounts.to);
    infos.push(accounts.authority);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
