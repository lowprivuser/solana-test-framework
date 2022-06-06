use async_trait::async_trait;
use borsh::BorshDeserialize;
use solana_program::program_pack::Pack;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{
        Keypair,
        Signer
    },
    system_transaction,
    sysvar::rent::Rent,
    transaction::Transaction,
    transport
};

use spl_associated_token_account::{
    get_associated_token_address,
    create_associated_token_account
};

#[cfg(feature = "anchor")]
use anchor_lang::AccountDeserialize;

use futures::{
    Future,
    FutureExt
};

use std::pin::Pin;

pub use {
    solana_banks_client::{BanksClient, BanksClientError},
};

/// Convenience functions for BanksClient
#[async_trait]
pub trait BanksClientExtensions {
    /// Assemble the given instructions into a transaction and sign it.
    /// All transactions created with this method are signed and payed for by the payer.
    async fn transaction_from_instructions(
        &mut self,
        _ixs: &[Instruction],
        _payer: &Keypair,
        _signers: Vec<&Keypair>
    ) -> Result<Transaction, BanksClientError> {
        unimplemented!();
    }

    /// Return and deserialize an Anchor account at the given address at the time of the most recent root slot.
    /// If the account is not found, `None` is returned.
    #[cfg(feature = "anchor")]
    fn get_account_with_anchor<T: AccountDeserialize>(
        &mut self,
        _address: Pubkey
    ) -> Pin<Box<dyn Future<Output = Result<T, BanksClientError>> + '_>> {
        unimplemented!();
    }

    /// Return and deserialize a Borsh account at the given address at the time of the most recent root slot.
    /// If the account is not `found`, None is returned.
    fn get_account_with_borsh<T: BorshDeserialize>(
        &mut self,
        _address: Pubkey
    ) -> Pin<Box<dyn Future<Output = Result<T, BanksClientError>> + '_>> {
        unimplemented!();
    }

    /// Create a new account
    async fn create_account(
        &mut self,
        _from: &Keypair,
        _to: &Keypair,
        _lamports: u64,
        _space: u64,
        _owner: Pubkey
    ) -> transport::Result<()> {
        unimplemented!();
    }

    /// Create a new SPL Token Mint account
    async fn create_token_mint(
        &mut self,
        _mint: &Keypair,
        _authority: &Pubkey,
        _freeze_authority: Option<&Pubkey>,
        _decimals: u8,
        _payer: &Keypair
    ) -> transport::Result<()> {
        unimplemented!();
    }

    /// Create a new SPL Token Account
    async fn create_token_account(
        &mut self,
        _account: &Keypair,
        _authority: &Pubkey,
        _mint: &Pubkey,
        _payer: &Keypair
    ) -> transport::Result<()> {
        unimplemented!();
    }

    /// Create a new SPL Associated Token Account
    async fn create_associated_token_account(
        &mut self,
        _authority: &Pubkey,
        _mint: &Pubkey,
        _payer: &Keypair
    ) -> transport::Result<Pubkey> {
        unimplemented!();
    }
}

#[async_trait]
impl BanksClientExtensions for BanksClient {
    async fn transaction_from_instructions(
        &mut self,
        ixs: &[Instruction],
        payer: &Keypair,
        signers: Vec<&Keypair>
    ) -> Result<Transaction, BanksClientError> {
        let latest_blockhash = self.get_latest_blockhash().await?;

        Ok(
            Transaction::new_signed_with_payer(
                ixs,
                Some(&payer.pubkey()),
                &signers,
                latest_blockhash
            )
        )
    }

    #[cfg(feature = "anchor")]
    fn get_account_with_anchor<T: AccountDeserialize>(
        &mut self,
        address: Pubkey,
    ) -> Pin<Box<dyn Future<Output = Result<T, BanksClientError>> + '_>> {
        Box::pin(self.get_account(address).map(|result| {
            let account = result?.ok_or(BanksClientError::ClientError("Account not found"))?;
            T::try_deserialize(&mut account.data.as_ref())
                .map_err(|_| BanksClientError::ClientError("Failed to deserialize account"))
        }))
    }

    fn get_account_with_borsh<T: BorshDeserialize>(
        &mut self,
        address: Pubkey,
    ) -> Pin<Box<dyn Future<Output = Result<T, BanksClientError>> + '_>> {
        Box::pin(self.get_account(address).map(|result| {
            let account = result?.ok_or(BanksClientError::ClientError("Account not found"))?;
            T::try_from_slice(&mut account.data.as_ref())
                .map_err(|_| BanksClientError::ClientError("Failed to deserialize account"))
        }))
    }

    async fn create_account(
        &mut self,
        from: &Keypair,
        to: &Keypair,
        lamports: u64,
        space: u64,
        owner: Pubkey
    ) -> transport::Result<()> {
        let latest_blockhash = self.get_latest_blockhash().await?;

        self.process_transaction(
            system_transaction::create_account(
                from,
                to,
                latest_blockhash,
                lamports,
                space,
                &owner
            )
        ).await
        .map_err(Into::into)
    }

    async fn create_token_mint(
        &mut self,
        mint: &Keypair,
        authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        decimals: u8,
        payer: &Keypair
    ) -> transport::Result<()> {
        let latest_blockhash = self.get_latest_blockhash().await?;
        self.process_transaction(system_transaction::create_account(
            &payer,
            &mint,
            latest_blockhash,
            Rent::default().minimum_balance(spl_token::state::Mint::get_packed_len()),
            spl_token::state::Mint::get_packed_len() as u64,
            &spl_token::id()
        )).await.unwrap();

        let ix = spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint.pubkey(),
            authority,
            freeze_authority,
            decimals,
        )
        .unwrap();

        self.process_transaction(
            Transaction::new_signed_with_payer(
                &[ix],
                Some(&payer.pubkey()),
                &[payer],
                latest_blockhash
            )
        ).await
        .map_err(Into::into)
    }

    async fn create_token_account(
        &mut self,
        account: &Keypair,
        authority: &Pubkey,
        mint: &Pubkey,
        payer: &Keypair
    ) -> transport::Result<()> {
        let latest_blockhash = self.get_latest_blockhash().await?;
        self.process_transaction(system_transaction::create_account(
            &payer,
            &account,
            latest_blockhash,
            Rent::default().minimum_balance(spl_token::state::Account::get_packed_len()),
            spl_token::state::Account::get_packed_len() as u64,
            &spl_token::id()
        )).await.unwrap();

        let ix = spl_token::instruction::initialize_account(
            &spl_token::id(),
            &account.pubkey(),
            mint,
            authority
        )
        .unwrap();

        self.process_transaction(
            Transaction::new_signed_with_payer(
                &[ix],
                Some(&payer.pubkey()),
                &[payer],
                latest_blockhash
            )
        ).await
        .map_err(Into::into)
    }

    async fn create_associated_token_account(
        &mut self,
        account: &Pubkey,
        mint: &Pubkey,
        payer: &Keypair
    ) -> transport::Result<Pubkey>  {
        let latest_blockhash = self.get_latest_blockhash().await?;
        let associated_token_account = get_associated_token_address(account, mint);
        let ix = create_associated_token_account(
            &payer.pubkey(), 
            &account, 
            &mint
        );

        self.process_transaction(
            Transaction::new_signed_with_payer(
                &[ix],
                Some(&payer.pubkey()),
                &[payer],
                latest_blockhash
            )
        ).await?;

        return Ok(associated_token_account);
    }
}