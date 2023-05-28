use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Program, Token2022};
use crate::prelude::FankorResult;

pub struct CpiMintToChecked<'info> {
    pub mint: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}

pub fn mint_to_checked(
    program: &Program<Token2022>,
    accounts: CpiMintToChecked,
    amount: u64,
    decimals: u8,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token_2022::instruction::mint_to_checked(
        program.address(),
        accounts.mint.key,
        accounts.to.key,
        accounts.authority.key,
        &[],
        amount,
        decimals,
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

pub struct CpiMintToCheckedMultisig<'info> {
    pub mint: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn mint_to_checked_multisig(
    program: &Program<Token2022>,
    accounts: CpiMintToCheckedMultisig,
    amount: u64,
    decimals: u8,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token_2022::instruction::mint_to_checked(
        program.address(),
        accounts.mint.key,
        accounts.to.key,
        accounts.authority.key,
        &signer_pubkeys,
        amount,
        decimals,
    )?;

    let mut infos = Vec::with_capacity(3 + accounts.signers.len());
    infos.push(accounts.mint);
    infos.push(accounts.to);
    infos.push(accounts.authority);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
