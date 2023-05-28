use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Program, Token2022};
use crate::prelude::FankorResult;

pub struct CpiRevoke<'info> {
    pub source: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
}

pub fn revoke(
    program: &Program<Token2022>,
    accounts: CpiRevoke,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token_2022::instruction::revoke(
        program.address(),
        accounts.source.key,
        accounts.owner.key,
        &[],
    )?;

    solana_program::program::invoke_signed(&ix, &[accounts.source, accounts.owner], signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiRevokeMultisig<'info> {
    pub source: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn revoke_multisig(
    program: &Program<Token2022>,
    accounts: CpiRevokeMultisig,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token_2022::instruction::revoke(
        program.address(),
        accounts.source.key,
        accounts.owner.key,
        &signer_pubkeys,
    )?;

    let mut infos = Vec::with_capacity(2 + accounts.signers.len());
    infos.push(accounts.source);
    infos.push(accounts.owner);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
