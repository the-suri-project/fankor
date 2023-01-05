use crate::cpi;
use crate::cpi::system_program::CpiCreateAccount;
use crate::cpi::token_2022::{CpiInitializeAccount3, CpiInitializeMint2, CpiInitializeMultisig2};
use crate::errors::FankorResult;
use crate::models::programs::macros::impl_account;
use crate::models::{Account, Program, System, UninitializedAccount};
use crate::traits::{AccountDeserialize, AccountSerialize, ProgramType};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use std::io::{ErrorKind, Write};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct Token2022;

impl ProgramType for Token2022 {
    fn name() -> &'static str {
        "Token2022"
    }

    fn address() -> &'static Pubkey {
        &spl_token_2022::ID
    }
}

// ----------------------------------------------------------------------------
// ACCOUNTS -------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl_account!(
    Mint2022,
    spl_token_2022::state::Mint,
    &spl_token_2022::ID,
    unpack,
    unpack,
    [Default]
);

impl_account!(
    TokenAccount2022,
    spl_token_2022::state::Account,
    &spl_token_2022::ID,
    unpack,
    unpack,
    [Default]
);

impl_account!(
    TokenMultisig2022,
    spl_token_2022::state::Multisig,
    &spl_token_2022::ID,
    unpack,
    unpack,
    [Default]
);

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl Mint2022 {
    // STATIC METHODS ---------------------------------------------------------

    /// Initializes a Mint account.
    pub fn init<'info>(
        account_to_init: UninitializedAccount<'info>,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token2022>,
    ) -> FankorResult<Account<'info, Mint2022>> {
        let rent = Rent::get()?;
        let space = spl_token_2022::state::Mint::LEN;
        let lamports = rent.minimum_balance(space);
        let account_to_init_info = account_to_init.info();

        cpi::system_program::create_account(
            system_program,
            CpiCreateAccount {
                from: payer,
                to: account_to_init_info.clone(),
            },
            lamports,
            space as u64,
            token_program.address(),
            &[],
        )?;

        cpi::token_2022::initialize_mint2(
            token_program,
            CpiInitializeMint2 {
                mint: account_to_init_info.clone(),
            },
            decimals,
            mint_authority,
            freeze_authority,
            &[],
        )?;

        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            Mint2022::try_deserialize(&mut data)?,
        )
    }

    /// Initializes a Mint account in a PDA.
    pub fn init_pda<'info>(
        account_to_init: UninitializedAccount<'info>,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token2022>,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, Mint2022>> {
        let rent = Rent::get()?;
        let space = spl_token_2022::state::Mint::LEN;
        let lamports = rent.minimum_balance(space);
        let account_to_init_info = account_to_init.info();

        cpi::system_program::create_account(
            system_program,
            CpiCreateAccount {
                from: payer,
                to: account_to_init_info.clone(),
            },
            lamports,
            space as u64,
            token_program.address(),
            &[seeds],
        )?;

        cpi::token_2022::initialize_mint2(
            token_program,
            CpiInitializeMint2 {
                mint: account_to_init_info.clone(),
            },
            decimals,
            mint_authority,
            freeze_authority,
            &[],
        )?;

        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            Mint2022::try_deserialize(&mut data)?,
        )
    }
}

impl TokenAccount2022 {
    // STATIC METHODS ---------------------------------------------------------

    /// Initializes a TokenAccount.
    pub fn init<'info>(
        account_to_init: UninitializedAccount<'info>,
        owner: &Pubkey,
        mint: AccountInfo<'info>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token2022>,
    ) -> FankorResult<Account<'info, TokenAccount2022>> {
        let rent = Rent::get()?;
        let space = spl_token_2022::state::Account::LEN;
        let lamports = rent.minimum_balance(space);
        let account_to_init_info = account_to_init.info();

        cpi::system_program::create_account(
            system_program,
            CpiCreateAccount {
                from: payer,
                to: account_to_init_info.clone(),
            },
            lamports,
            space as u64,
            token_program.address(),
            &[],
        )?;

        cpi::token_2022::initialize_account3(
            token_program,
            CpiInitializeAccount3 {
                account: account_to_init_info.clone(),
                mint,
            },
            owner,
            &[],
        )?;

        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            TokenAccount2022::try_deserialize(&mut data)?,
        )
    }

    /// Initializes a TokenAccount in a PDA.
    pub fn init_pda<'info>(
        account_to_init: UninitializedAccount<'info>,
        owner: &Pubkey,
        mint: AccountInfo<'info>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token2022>,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, TokenAccount2022>> {
        let rent = Rent::get()?;
        let space = spl_token_2022::state::Account::LEN;
        let lamports = rent.minimum_balance(space);
        let account_to_init_info = account_to_init.info();

        cpi::system_program::create_account(
            system_program,
            CpiCreateAccount {
                from: payer,
                to: account_to_init_info.clone(),
            },
            lamports,
            space as u64,
            token_program.address(),
            &[seeds],
        )?;

        cpi::token_2022::initialize_account3(
            token_program,
            CpiInitializeAccount3 {
                account: account_to_init_info.clone(),
                mint,
            },
            owner,
            &[],
        )?;

        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            TokenAccount2022::try_deserialize(&mut data)?,
        )
    }
}

impl TokenMultisig2022 {
    // STATIC METHODS ---------------------------------------------------------

    /// Initializes a TokenMultisig account.
    pub fn init<'info>(
        account_to_init: UninitializedAccount<'info>,
        m: u8,
        signers: Vec<AccountInfo<'info>>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token2022>,
    ) -> FankorResult<Account<'info, TokenMultisig2022>> {
        let rent = Rent::get()?;
        let space = spl_token_2022::state::Multisig::LEN;
        let lamports = rent.minimum_balance(space);
        let account_to_init_info = account_to_init.info();

        cpi::system_program::create_account(
            system_program,
            CpiCreateAccount {
                from: payer,
                to: account_to_init_info.clone(),
            },
            lamports,
            space as u64,
            token_program.address(),
            &[],
        )?;

        cpi::token_2022::initialize_multisig2(
            token_program,
            CpiInitializeMultisig2 {
                multisignature: account_to_init_info.clone(),
                signers,
            },
            m,
            &[],
        )?;

        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            TokenMultisig2022::try_deserialize(&mut data)?,
        )
    }

    /// Initializes a TokenMultisig account in a PDA.
    pub fn init_pda<'info>(
        account_to_init: UninitializedAccount<'info>,
        m: u8,
        signers: Vec<AccountInfo<'info>>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token2022>,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, TokenMultisig2022>> {
        let rent = Rent::get()?;
        let space = spl_token_2022::state::Multisig::LEN;
        let lamports = rent.minimum_balance(space);
        let account_to_init_info = account_to_init.info();

        cpi::system_program::create_account(
            system_program,
            CpiCreateAccount {
                from: payer,
                to: account_to_init_info.clone(),
            },
            lamports,
            space as u64,
            token_program.address(),
            &[seeds],
        )?;

        cpi::token_2022::initialize_multisig2(
            token_program,
            CpiInitializeMultisig2 {
                multisignature: account_to_init_info.clone(),
                signers,
            },
            m,
            &[],
        )?;

        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            TokenMultisig2022::try_deserialize(&mut data)?,
        )
    }
}
