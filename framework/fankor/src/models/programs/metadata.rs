use crate::cpi;
use crate::cpi::metadata::{CpiCreateMasterEditionV3, CpiCreateMetadataAccountV3};
use crate::cpi::system_program::CpiCreateAccount;
use crate::errors::FankorResult;
use crate::models::programs::macros::impl_account;
use crate::models::{Account, Program, System, Token, UninitializedAccount};
use crate::traits::ProgramType;
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::state::{
    Collection, CollectionDetails, Creator, TokenMetadataAccount, Uses, BURN, COLLECTION_AUTHORITY,
    EDITION, PREFIX, USER,
};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use std::io::{ErrorKind, Write};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct Metadata;

impl ProgramType for Metadata {
    fn name() -> &'static str {
        "Metadata"
    }

    fn address() -> &'static Pubkey {
        &mpl_token_metadata::ID
    }
}

impl Metadata {
    // METHODS ----------------------------------------------------------------

    pub fn get_edition_pda_seeds<'a>(mint: &'a Pubkey, edition_number: &'a str) -> [&'a [u8]; 5] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            EDITION.as_bytes(),
            edition_number.as_bytes(),
        ]
    }

    pub fn get_master_edition_pda_seeds(mint: &Pubkey) -> [&[u8]; 4] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            EDITION.as_bytes(),
        ]
    }

    pub fn get_metadata_pda_seeds(mint: &Pubkey) -> [&[u8]; 3] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
        ]
    }

    pub fn get_use_authority_pda_seeds<'a>(
        mint: &'a Pubkey,
        authority: &'a Pubkey,
    ) -> [&'a [u8]; 5] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            USER.as_bytes(),
            authority.as_ref(),
        ]
    }

    pub fn get_collection_pda_seeds<'a>(mint: &'a Pubkey, authority: &'a Pubkey) -> [&'a [u8]; 5] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            COLLECTION_AUTHORITY.as_bytes(),
            authority.as_ref(),
        ]
    }

    pub fn get_program_as_burner_pda_seeds<'a>() -> [&'a [u8]; 3] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            BURN.as_bytes(),
        ]
    }
}

// ----------------------------------------------------------------------------
// ACCOUNTS -------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl_account!(
    meta: MetadataAccount,
    mpl_token_metadata::state::Metadata,
    &mpl_token_metadata::ID,
);

impl_account!(
    meta: MasterEditionV1,
    mpl_token_metadata::state::MasterEditionV1,
    &mpl_token_metadata::ID,
);

impl_account!(
    meta: MasterEditionV2,
    mpl_token_metadata::state::MasterEditionV2,
    &mpl_token_metadata::ID,
);

impl_account!(
    meta: Edition,
    mpl_token_metadata::state::Edition,
    &mpl_token_metadata::ID,
);

impl_account!(
    meta: ReservationListV1,
    mpl_token_metadata::state::ReservationListV1,
    &mpl_token_metadata::ID,
);

impl_account!(
    meta: ReservationListV2,
    mpl_token_metadata::state::ReservationListV2,
    &mpl_token_metadata::ID,
);

impl_account!(
    meta: EditionMarker,
    mpl_token_metadata::state::EditionMarker,
    &mpl_token_metadata::ID,
);

impl_account!(
    meta: UseAuthorityRecord,
    mpl_token_metadata::state::UseAuthorityRecord,
    &mpl_token_metadata::ID,
);

impl_account!(
    meta: CollectionAuthorityRecord,
    mpl_token_metadata::state::CollectionAuthorityRecord,
    &mpl_token_metadata::ID,
);

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl MetadataAccount {
    // STATIC METHODS ---------------------------------------------------------

    /// Initializes a Mint account.
    #[allow(clippy::too_many_arguments)]
    pub fn init<'info>(
        account_to_init: UninitializedAccount<'info>,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        update_authority_is_signer: bool,
        is_mutable: bool,
        collection: Option<Collection>,
        uses: Option<Uses>,
        collection_details: Option<CollectionDetails>,
        mint: AccountInfo<'info>,
        mint_authority: AccountInfo<'info>,
        update_authority: AccountInfo<'info>,
        payer: AccountInfo<'info>,
        system_program: &Program<'info, System>,
        metadata_program: &Program<'info, Metadata>,
        rent_sysvar: AccountInfo<'info>,
    ) -> FankorResult<Account<'info, MetadataAccount>> {
        let rent = Rent::get()?;
        let space = mpl_token_metadata::state::Metadata::size();
        let lamports = rent.minimum_balance(space);
        let account_to_init_info = account_to_init.info();

        cpi::system_program::create_account(
            system_program,
            CpiCreateAccount {
                from: payer.clone(),
                to: account_to_init_info.clone(),
            },
            lamports,
            space as u64,
            metadata_program.address(),
            &[],
        )?;

        cpi::metadata::create_metadata_accounts_v3(
            metadata_program,
            CpiCreateMetadataAccountV3 {
                metadata: account_to_init_info.clone(),
                mint,
                mint_authority,
                payer,
                update_authority,
                system_program: system_program.info().clone(),
                rent_sysvar,
            },
            name,
            symbol,
            uri,
            creators,
            seller_fee_basis_points,
            update_authority_is_signer,
            is_mutable,
            collection,
            uses,
            collection_details,
            &[],
        )?;

        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            MetadataAccount::deserialize(&mut data)?,
        )
    }

    /// Initializes a Mint account in a PDA.
    #[allow(clippy::too_many_arguments)]
    pub fn init_pda<'info>(
        account_to_init: UninitializedAccount<'info>,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        update_authority_is_signer: bool,
        is_mutable: bool,
        collection: Option<Collection>,
        uses: Option<Uses>,
        collection_details: Option<CollectionDetails>,
        mint: AccountInfo<'info>,
        mint_authority: AccountInfo<'info>,
        update_authority: AccountInfo<'info>,
        payer: AccountInfo<'info>,
        system_program: &Program<'info, System>,
        metadata_program: &Program<'info, Metadata>,
        rent_sysvar: AccountInfo<'info>,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, MetadataAccount>> {
        let rent = Rent::get()?;
        let space = mpl_token_metadata::state::Metadata::size();
        let lamports = rent.minimum_balance(space);
        let account_to_init_info = account_to_init.info();

        cpi::system_program::create_account(
            system_program,
            CpiCreateAccount {
                from: payer.clone(),
                to: account_to_init_info.clone(),
            },
            lamports,
            space as u64,
            metadata_program.address(),
            &[seeds],
        )?;

        cpi::metadata::create_metadata_accounts_v3(
            metadata_program,
            CpiCreateMetadataAccountV3 {
                metadata: account_to_init_info.clone(),
                mint,
                mint_authority,
                payer,
                update_authority,
                system_program: system_program.info().clone(),
                rent_sysvar,
            },
            name,
            symbol,
            uri,
            creators,
            seller_fee_basis_points,
            update_authority_is_signer,
            is_mutable,
            collection,
            uses,
            collection_details,
            &[],
        )?;

        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            MetadataAccount::deserialize(&mut data)?,
        )
    }
}

impl MasterEditionV2 {
    // STATIC METHODS ---------------------------------------------------------

    /// Initializes a Mint account.
    #[allow(clippy::too_many_arguments)]
    pub fn init<'info>(
        account_to_init: UninitializedAccount<'info>,
        max_supply: Option<u64>,
        mint: AccountInfo<'info>,
        update_authority: AccountInfo<'info>,
        mint_authority: AccountInfo<'info>,
        metadata: AccountInfo<'info>,
        payer: AccountInfo<'info>,
        system_program: &Program<'info, System>,
        token_program: &Program<'info, Token>,
        metadata_program: &Program<'info, Metadata>,
        rent_sysvar: AccountInfo<'info>,
    ) -> FankorResult<Account<'info, MasterEditionV2>> {
        let rent = Rent::get()?;
        let space = mpl_token_metadata::state::MasterEditionV2::size();
        let lamports = rent.minimum_balance(space);
        let account_to_init_info = account_to_init.info();

        cpi::system_program::create_account(
            system_program,
            CpiCreateAccount {
                from: payer.clone(),
                to: account_to_init_info.clone(),
            },
            lamports,
            space as u64,
            metadata_program.address(),
            &[],
        )?;

        cpi::metadata::create_master_edition_v3(
            metadata_program,
            CpiCreateMasterEditionV3 {
                edition: account_to_init_info.clone(),
                mint,
                update_authority,
                mint_authority,
                metadata,
                payer,
                token_program: token_program.info().clone(),
                system_program: system_program.info().clone(),
                rent_sysvar,
            },
            max_supply,
            &[],
        )?;

        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            MasterEditionV2::deserialize(&mut data)?,
        )
    }

    /// Initializes a Mint account in a PDA.
    #[allow(clippy::too_many_arguments)]
    pub fn init_pda<'info>(
        account_to_init: UninitializedAccount<'info>,
        max_supply: Option<u64>,
        mint: AccountInfo<'info>,
        update_authority: AccountInfo<'info>,
        mint_authority: AccountInfo<'info>,
        metadata: AccountInfo<'info>,
        payer: AccountInfo<'info>,
        system_program: &Program<'info, System>,
        token_program: &Program<'info, Token>,
        metadata_program: &Program<'info, Metadata>,
        rent_sysvar: AccountInfo<'info>,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, MasterEditionV2>> {
        let rent = Rent::get()?;
        let space = mpl_token_metadata::state::MasterEditionV2::size();
        let lamports = rent.minimum_balance(space);
        let account_to_init_info = account_to_init.info();

        cpi::system_program::create_account(
            system_program,
            CpiCreateAccount {
                from: payer.clone(),
                to: account_to_init_info.clone(),
            },
            lamports,
            space as u64,
            metadata_program.address(),
            &[seeds],
        )?;

        cpi::metadata::create_master_edition_v3(
            metadata_program,
            CpiCreateMasterEditionV3 {
                edition: account_to_init_info.clone(),
                mint,
                update_authority,
                mint_authority,
                metadata,
                payer,
                token_program: token_program.info().clone(),
                system_program: system_program.info().clone(),
                rent_sysvar,
            },
            max_supply,
            &[],
        )?;

        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            MasterEditionV2::deserialize(&mut data)?,
        )
    }
}
