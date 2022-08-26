use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiBurnNft<'info> {
    pub metadata: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub token: AccountInfo<'info>,
    pub edition: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub collection_metadata: Option<AccountInfo<'info>>,
}

pub fn burn_nft(
    program: &Program<Metadata>,
    accounts: CpiBurnNft,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = mpl_token_metadata::instruction::burn_nft(
        *program.address(),
        *accounts.metadata.key,
        *accounts.owner.key,
        *accounts.mint.key,
        *accounts.token.key,
        *accounts.edition.key,
        *accounts.token_program.key,
        accounts.collection_metadata.as_ref().map(|a| *a.key),
    );

    let mut infos = Vec::with_capacity(7);
    infos.push(accounts.metadata);
    infos.push(accounts.owner);
    infos.push(accounts.mint);
    infos.push(accounts.token);
    infos.push(accounts.edition);
    infos.push(accounts.token_program);

    if let Some(collection_metadata) = accounts.collection_metadata {
        infos.push(collection_metadata);
    }

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
