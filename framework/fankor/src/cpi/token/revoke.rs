use crate::errors::Error;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiRevoke<'info> {
    pub source: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
}

pub fn revoke(accounts: CpiRevoke, signer_seeds: &[&[&[u8]]]) -> FankorResult<()> {
    let ix = spl_token::instruction::revoke(
        &spl_token::ID,
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

pub fn revoke_multisig(accounts: CpiRevokeMultisig, signer_seeds: &[&[&[u8]]]) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token::instruction::revoke(
        &spl_token::ID,
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
