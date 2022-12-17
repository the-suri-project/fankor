//! Code based on https://github.com/halbornteam/solana-test-framework

use borsh::BorshSerialize;
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program_test::ProgramTest;
use solana_sdk::account::Account;
use spl_associated_token_account::get_associated_token_address;

pub trait ProgramTestExtension {
    /// Adds an account with some Borsh-serializable to the test environment.
    fn add_account_with_value<B: BorshSerialize>(
        &mut self,
        pubkey: Pubkey,
        owner: Pubkey,
        lamports: u64,
        data: B,
    );

    /// Adds a rent-exempt account to the test environment.
    fn add_rent_exempt_account(&mut self, pubkey: Pubkey, owner: Pubkey, data: Vec<u8>);

    /// Adds a rent-exempt account with some Borsh-serializable to the test environment.
    fn add_rent_exempt_account_with_value<B: BorshSerialize>(
        &mut self,
        pubkey: Pubkey,
        owner: Pubkey,
        data: B,
    );

    /// Adds an SPL Token Mint account to the test environment.
    fn add_token_mint(
        &mut self,
        pubkey: Pubkey,
        mint_authority: Option<Pubkey>,
        supply: u64,
        decimals: u8,
        freeze_authority: Option<Pubkey>,
    );

    /// Adds an SPL Token account to the test environment.
    #[allow(clippy::too_many_arguments)]
    fn add_token_account(
        &mut self,
        pubkey: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
        delegate: Option<Pubkey>,
        is_native: Option<u64>,
        delegated_amount: u64,
        close_authority: Option<Pubkey>,
    );

    /// Adds an associated token account to the test environment.
    /// Returns the address of the created account.
    #[allow(clippy::too_many_arguments)]
    fn add_associated_token_account(
        &mut self,
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
        delegate: Option<Pubkey>,
        is_native: Option<u64>,
        delegated_amount: u64,
        close_authority: Option<Pubkey>,
    ) -> Pubkey;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl ProgramTestExtension for ProgramTest {
    fn add_account_with_value<B: BorshSerialize>(
        &mut self,
        pubkey: Pubkey,
        owner: Pubkey,
        lamports: u64,
        data: B,
    ) {
        self.add_account(
            pubkey,
            Account {
                lamports,
                data: data.try_to_vec().expect("failed to serialize data"),
                owner,
                executable: false,
                rent_epoch: 0,
            },
        );
    }

    fn add_rent_exempt_account(&mut self, pubkey: Pubkey, owner: Pubkey, data: Vec<u8>) {
        self.add_account(
            pubkey,
            Account {
                lamports: Rent::default().minimum_balance(data.len()),
                data: data.try_to_vec().expect("failed to serialize data"),
                owner,
                executable: false,
                rent_epoch: 0,
            },
        );
    }

    fn add_rent_exempt_account_with_value<B: BorshSerialize>(
        &mut self,
        pubkey: Pubkey,
        owner: Pubkey,
        data: B,
    ) {
        ProgramTestExtension::add_rent_exempt_account(
            self,
            pubkey,
            owner,
            data.try_to_vec().expect("failed to serialize data"),
        );
    }

    fn add_token_mint(
        &mut self,
        pubkey: Pubkey,
        mint_authority: Option<Pubkey>,
        supply: u64,
        decimals: u8,
        freeze_authority: Option<Pubkey>,
    ) {
        let data = spl_token::state::Mint {
            mint_authority: COption::from(mint_authority),
            supply,
            decimals,
            is_initialized: true,
            freeze_authority: COption::from(freeze_authority),
        };

        let data = {
            let mut buf = vec![0u8; spl_token::state::Mint::LEN];
            data.pack_into_slice(&mut buf[..]);
            buf
        };
        self.add_rent_exempt_account(pubkey, spl_token::id(), data);
    }

    fn add_token_account(
        &mut self,
        pubkey: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
        delegate: Option<Pubkey>,
        is_native: Option<u64>,
        delegated_amount: u64,
        close_authority: Option<Pubkey>,
    ) {
        let data = spl_token::state::Account {
            mint,
            owner,
            amount,
            delegate: COption::from(delegate),
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::from(is_native),
            delegated_amount,
            close_authority: COption::from(close_authority),
        };

        let data = {
            let mut buf = vec![0u8; spl_token::state::Mint::LEN];
            data.pack_into_slice(&mut buf[..]);
            buf
        };
        self.add_rent_exempt_account(pubkey, spl_token::id(), data);
    }

    fn add_associated_token_account(
        &mut self,
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
        delegate: Option<Pubkey>,
        is_native: Option<u64>,
        delegated_amount: u64,
        close_authority: Option<Pubkey>,
    ) -> Pubkey {
        let pubkey = get_associated_token_address(&owner, &mint);
        self.add_token_account(
            pubkey,
            mint,
            owner,
            amount,
            delegate,
            is_native,
            delegated_amount,
            close_authority,
        );

        pubkey
    }
}
