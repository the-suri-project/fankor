use std::io::{ErrorKind, Write};
use std::ops::Deref;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use crate::cpi;
use crate::cpi::associated_token::CpiCreateAssociatedTokenAccount;
use crate::cpi::system_program::CpiCreateAccount;
use crate::cpi::token::{CpiInitializeAccount3, CpiInitializeMint2, CpiInitializeMultisig2};
use crate::errors::FankorResult;
use crate::models::programs::macros::impl_account;
use crate::models::{Account, AssociatedToken, Program, System, UninitializedAccount};
use crate::traits::ProgramType;

#[derive(Debug, Copy, Clone)]
pub struct Token;

impl ProgramType for Token {
    fn name() -> &'static str {
        "Token"
    }

    fn address() -> &'static Pubkey {
        &spl_token::ID
    }
}

// ----------------------------------------------------------------------------
// ACCOUNTS -------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl_account!(token: Mint, spl_token::state::Mint, &spl_token::ID);

impl_account!(
    token: TokenAccount,
    spl_token::state::Account,
    &spl_token::ID,
);

impl_account!(
    token: TokenMultisig,
    spl_token::state::Multisig,
    &spl_token::ID,
);

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl Mint {
    // STATIC METHODS ---------------------------------------------------------

    /// Initializes a Mint account.
    pub fn init<'info>(
        account_to_init: UninitializedAccount<'info>,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token>,
    ) -> FankorResult<Account<'info, Mint>> {
        let rent = Rent::get()?;
        let space = spl_token::state::Mint::LEN;
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

        cpi::token::initialize_mint2(
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
            Mint::deserialize(&mut data)?,
        )
    }

    /// Initializes a Mint account in a PDA.
    #[allow(clippy::too_many_arguments)]
    pub fn init_pda<'info>(
        account_to_init: UninitializedAccount<'info>,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token>,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, Mint>> {
        let rent = Rent::get()?;
        let space = spl_token::state::Mint::LEN;
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

        cpi::token::initialize_mint2(
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
            Mint::deserialize(&mut data)?,
        )
    }
}

impl TokenAccount {
    // STATIC METHODS ---------------------------------------------------------

    /// Initializes a TokenAccount.
    pub fn init<'info>(
        account_to_init: UninitializedAccount<'info>,
        owner: &Pubkey,
        mint: AccountInfo<'info>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token>,
    ) -> FankorResult<Account<'info, TokenAccount>> {
        let rent = Rent::get()?;
        let space = spl_token::state::Account::LEN;
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

        cpi::token::initialize_account3(
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
            TokenAccount::deserialize(&mut data)?,
        )
    }

    /// Initializes a TokenAccount in a PDA.
    pub fn init_pda<'info>(
        account_to_init: UninitializedAccount<'info>,
        owner: &Pubkey,
        mint: AccountInfo<'info>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token>,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, TokenAccount>> {
        let rent = Rent::get()?;
        let space = spl_token::state::Account::LEN;
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

        cpi::token::initialize_account3(
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
            TokenAccount::deserialize(&mut data)?,
        )
    }

    /// Initializes a TokenAccount in an associated token account.
    #[allow(clippy::too_many_arguments)]
    pub fn init_associated<'info>(
        account_to_init: UninitializedAccount<'info>,
        owner: AccountInfo<'info>,
        mint: AccountInfo<'info>,
        payer: AccountInfo<'info>,
        _system_program: &Program<System>,
        token_program: &Program<'info, Token>,
        associated_token_program: &Program<AssociatedToken>,
        seeds: &[&[&[u8]]],
    ) -> FankorResult<Account<'info, TokenAccount>> {
        cpi::associated_token::create_associated_token_account(
            associated_token_program,
            CpiCreateAssociatedTokenAccount {
                funding_address: payer,
                wallet_address: owner,
                token_mint_address: mint,
                token_program: token_program.info().clone(),
            },
            seeds,
        )?;

        let account_to_init_info = account_to_init.info();
        let mut data: &[u8] = &account_to_init_info.try_borrow_data()?;
        Account::new(
            account_to_init.context(),
            account_to_init_info,
            TokenAccount::deserialize(&mut data)?,
        )
    }
}

impl TokenMultisig {
    // STATIC METHODS ---------------------------------------------------------

    /// Initializes a TokenMultisig account.
    pub fn init<'info>(
        account_to_init: UninitializedAccount<'info>,
        m: u8,
        signers: Vec<AccountInfo<'info>>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token>,
    ) -> FankorResult<Account<'info, TokenMultisig>> {
        let rent = Rent::get()?;
        let space = spl_token::state::Multisig::LEN;
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

        cpi::token::initialize_multisig2(
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
            TokenMultisig::deserialize(&mut data)?,
        )
    }

    /// Initializes a TokenMultisig account in a PDA.
    pub fn init_pda<'info>(
        account_to_init: UninitializedAccount<'info>,
        m: u8,
        signers: Vec<AccountInfo<'info>>,
        payer: AccountInfo<'info>,
        system_program: &Program<System>,
        token_program: &Program<Token>,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, TokenMultisig>> {
        let rent = Rent::get()?;
        let space = spl_token::state::Multisig::LEN;
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

        cpi::token::initialize_multisig2(
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
            TokenMultisig::deserialize(&mut data)?,
        )
    }
}
