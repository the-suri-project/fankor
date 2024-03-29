use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::FankorResult;

pub struct CpiSetAndVerifySizedCollectionItem<'info> {
    pub metadata: AccountInfo<'info>,
    pub collection_authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub update_authority: AccountInfo<'info>,
    pub collection_mint: AccountInfo<'info>,
    pub collection: AccountInfo<'info>,
    pub collection_master_edition_account: AccountInfo<'info>,
    pub collection_authority_record: Option<AccountInfo<'info>>,
}

pub fn set_and_verify_sized_collection_item(
    program: &Program<Metadata>,
    accounts: CpiSetAndVerifySizedCollectionItem,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = mpl_token_metadata::instruction::set_and_verify_sized_collection_item(
        *program.address(),
        *accounts.metadata.key,
        *accounts.collection_authority.key,
        *accounts.payer.key,
        *accounts.update_authority.key,
        *accounts.collection_mint.key,
        *accounts.collection.key,
        *accounts.collection_master_edition_account.key,
        accounts
            .collection_authority_record
            .as_ref()
            .map(|a| *a.key),
    );

    let mut infos = Vec::with_capacity(8);
    infos.push(accounts.metadata);
    infos.push(accounts.collection_authority);
    infos.push(accounts.payer);
    infos.push(accounts.update_authority);
    infos.push(accounts.collection_mint);
    infos.push(accounts.collection);
    infos.push(accounts.collection_master_edition_account);

    if let Some(collection_authority_record) = accounts.collection_authority_record {
        infos.push(collection_authority_record);
    }

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
