use crate::errors::Error;
use crate::models::{Program, Token2022};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use spl_token_2022::instruction::AuthorityType;

pub struct CpiSetAuthority<'info> {
    pub owned: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
}

pub fn set_authority(
    program: &Program<Token2022>,
    accounts: CpiSetAuthority,
    authority_type: AuthorityType,
    new_authority: Option<&Pubkey>,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token_2022::instruction::set_authority(
        program.address(),
        accounts.owned.key,
        new_authority,
        authority_type,
        accounts.owner.key,
        &[],
    )?;

    solana_program::program::invoke_signed(&ix, &[accounts.owned, accounts.owner], signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiSetAuthorityMultisig<'info> {
    pub owned: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn set_authority_multisig(
    program: &Program<Token2022>,
    accounts: CpiSetAuthorityMultisig,
    authority_type: AuthorityType,
    new_authority: Option<&Pubkey>,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token_2022::instruction::set_authority(
        program.address(),
        accounts.owned.key,
        new_authority,
        authority_type,
        accounts.owner.key,
        &signer_pubkeys,
    )?;

    let mut infos = Vec::with_capacity(2 + accounts.signers.len());
    infos.push(accounts.owned);
    infos.push(accounts.owner);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
