//! Code based on https://github.com/halbornteam/solana-test-framework

use async_trait::async_trait;
use borsh::BorshDeserialize;
use solana_program::program_pack::Pack;
use solana_program_test::{BanksClient, BanksClientError};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_transaction,
    sysvar::rent::Rent,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;

/// Convenience functions for clients
#[async_trait]
pub trait ClientExtensions {
    /// Assemble the given instructions into a transaction and sign it.
    /// All transactions created with this method are signed and payed for by the payer.
    async fn create_transaction_from_instructions(
        &mut self,
        _ixs: &[Instruction],
        _payer: &Keypair,
        _signers: Vec<&Keypair>,
    ) -> Result<Transaction, Box<dyn std::error::Error>> {
        unimplemented!();
    }

    /// Create a new account.
    async fn create_account(
        &mut self,
        _from: &Keypair,
        _to: &Keypair,
        _lamports: u64,
        _space: u64,
        _owner: Pubkey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unimplemented!();
    }

    /// Create a new SPL Token Mint account.
    async fn create_token_mint(
        &mut self,
        _mint: &Keypair,
        _authority: &Pubkey,
        _freeze_authority: Option<&Pubkey>,
        _decimals: u8,
        _payer: &Keypair,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unimplemented!();
    }

    /// Create a new SPL Token Account.
    async fn create_token_account(
        &mut self,
        _account: &Keypair,
        _authority: &Pubkey,
        _mint: &Pubkey,
        _payer: &Keypair,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unimplemented!();
    }

    /// Create a new SPL Associated Token Account.
    async fn create_associated_token_account(
        &mut self,
        _authority: &Pubkey,
        _mint: &Pubkey,
        _payer: &Keypair,
    ) -> Result<Pubkey, Box<dyn std::error::Error>> {
        unimplemented!();
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[async_trait]
impl ClientExtensions for BanksClient {
    async fn create_transaction_from_instructions(
        &mut self,
        ixs: &[Instruction],
        payer: &Keypair,
        signers: Vec<&Keypair>,
    ) -> Result<Transaction, Box<dyn std::error::Error>> {
        let latest_blockhash = self.get_latest_blockhash().await?;

        Ok(Transaction::new_signed_with_payer(
            ixs,
            Some(&payer.pubkey()),
            &signers,
            latest_blockhash,
        ))
    }

    async fn create_account(
        &mut self,
        from: &Keypair,
        to: &Keypair,
        lamports: u64,
        space: u64,
        owner: Pubkey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let latest_blockhash = self.get_latest_blockhash().await?;

        self.process_transaction(system_transaction::create_account(
            from,
            to,
            latest_blockhash,
            lamports,
            space,
            &owner,
        ))
        .await
        .map_err(Into::into)
    }

    async fn create_token_mint(
        &mut self,
        mint: &Keypair,
        authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        decimals: u8,
        payer: &Keypair,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let latest_blockhash = self.get_latest_blockhash().await?;
        self.process_transaction(system_transaction::create_account(
            payer,
            mint,
            latest_blockhash,
            Rent::default().minimum_balance(spl_token::state::Mint::get_packed_len()),
            spl_token::state::Mint::get_packed_len() as u64,
            &spl_token::id(),
        ))
        .await
        .unwrap();

        let ix = spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint.pubkey(),
            authority,
            freeze_authority,
            decimals,
        )
        .unwrap();

        self.process_transaction(Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[payer],
            latest_blockhash,
        ))
        .await
        .map_err(Into::into)
    }

    async fn create_token_account(
        &mut self,
        account: &Keypair,
        authority: &Pubkey,
        mint: &Pubkey,
        payer: &Keypair,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let latest_blockhash = self.get_latest_blockhash().await?;
        self.process_transaction(system_transaction::create_account(
            payer,
            account,
            latest_blockhash,
            Rent::default().minimum_balance(spl_token::state::Account::get_packed_len()),
            spl_token::state::Account::get_packed_len() as u64,
            &spl_token::id(),
        ))
        .await
        .unwrap();

        let ix = spl_token::instruction::initialize_account(
            &spl_token::id(),
            &account.pubkey(),
            mint,
            authority,
        )
        .unwrap();

        self.process_transaction(Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[payer],
            latest_blockhash,
        ))
        .await
        .map_err(Into::into)
    }

    async fn create_associated_token_account(
        &mut self,
        account: &Pubkey,
        mint: &Pubkey,
        payer: &Keypair,
    ) -> Result<Pubkey, Box<dyn std::error::Error>> {
        let latest_blockhash = self.get_latest_blockhash().await?;
        let associated_token_account = get_associated_token_address(account, mint);
        let ix = create_associated_token_account(&payer.pubkey(), account, mint, &spl_token::id());

        self.process_transaction(Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[payer],
            latest_blockhash,
        ))
        .await?;

        return Ok(associated_token_account);
    }
}
