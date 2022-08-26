use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiSetCollectionSize<'info> {
    pub metadata: AccountInfo<'info>,
    pub update_authority: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub collection_authority_record: Option<AccountInfo<'info>>,
}

pub fn set_collection_size(
    program: &Program<Metadata>,
    accounts: CpiSetCollectionSize,
    size: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = mpl_token_metadata::instruction::set_collection_size(
        *program.address(),
        *accounts.metadata.key,
        *accounts.update_authority.key,
        *accounts.mint.key,
        accounts
            .collection_authority_record
            .as_ref()
            .map(|e| *e.key),
        size,
    );

    let mut infos = Vec::with_capacity(4);
    infos.push(accounts.metadata);
    infos.push(accounts.update_authority);
    infos.push(accounts.mint);

    if let Some(edition) = accounts.collection_authority_record {
        infos.push(edition);
    }

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
