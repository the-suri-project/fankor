use crate::errors::Error;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiMintToChecked<'info> {
    pub mint: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}

pub fn mint_to_checked(
    accounts: CpiMintToChecked,
    amount: u64,
    decimals: u8,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token::instruction::mint_to_checked(
        &spl_token::ID,
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
    accounts: CpiMintToCheckedMultisig,
    amount: u64,
    decimals: u8,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token::instruction::mint_to_checked(
        &spl_token::ID,
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
