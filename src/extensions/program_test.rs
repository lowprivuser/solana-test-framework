use solana_program::{
    program_option::COption,
    program_pack::Pack
};
use solana_program_test::ProgramTest;
use solana_sdk::{
    account::Account,
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    sysvar::rent::Rent
};
use spl_associated_token_account::get_associated_token_address;
use borsh::BorshSerialize;

#[cfg(feature = "anchor")]
use anchor_lang::{AnchorSerialize, Discriminator};

pub trait ProgramTestExtension {
    /// Adds a requested number of account with initial balance of 1_000 SOL to the test environment
    fn generate_accounts(&mut self, number_of_accounts: u8) -> Vec<Keypair>;
    
    /// Add a rent-exempt account with some data to the test environment.
    fn add_account_with_data(&mut self, pubkey: Pubkey, owner: Pubkey, data: &[u8], executable: bool);
    
    #[cfg(feature = "anchor")]
    /// Adds an Anchor account.
    fn add_account_with_anchor<T: AnchorSerialize + Discriminator>(&mut self, pubkey: Pubkey, owner: Pubkey, anchor_data: T, executable: bool);
    
    /// Adds an account with the given balance to the test environment.
    fn add_account_with_lamports(&mut self, pubkey: Pubkey, owner: Pubkey, lamports: u64);
    
    /// Adds a rent-exempt account with some Packable data to the test environment.
    fn add_account_with_packable<P: Pack>(&mut self, pubkey: Pubkey, owner: Pubkey, data: P);

    /// Adds a rent-exempt account with some Borsh-serializable to the test environment
    fn add_account_with_borsh<B: BorshSerialize>(&mut self, pubkey: Pubkey, owner: Pubkey, data: B);
    
    /// Adds an SPL Token Mint account to the test environment.
    fn add_token_mint(&mut self, pubkey: Pubkey, mint_authority: Option<Pubkey>, supply: u64, decimals: u8, freeze_authority: Option<Pubkey>);
    
    /// Adds an SPL Token account to the test environment.
    fn add_token_account(&mut self, pubkey: Pubkey, mint: Pubkey, owner: Pubkey, amount: u64);
    
    // Adds an associated token account to the test environment.
    // Returns the address of the created account.
    fn add_associated_token_account(&mut self, owner: Pubkey, mint: Pubkey, amount: u64) -> Pubkey;
}

impl ProgramTestExtension for ProgramTest {
    fn generate_accounts(&mut self, number_of_accounts: u8) -> Vec<Keypair> {
        let mut accounts: Vec<Keypair> = vec![];

        for _ in 0..number_of_accounts {
            let keypair = Keypair::new();
            let initial_lamports = sol_to_lamports(1_000.0);
            self.add_account_with_lamports(keypair.pubkey(), keypair.pubkey(), initial_lamports);
            accounts.push(keypair);
        }
        accounts
    }

    fn add_account_with_data(
        &mut self,
        pubkey: Pubkey,
        owner: Pubkey,
        data: &[u8],
        executable: bool
    ) {
        self.add_account(
            pubkey,
            Account {
                lamports: Rent::default().minimum_balance(data.len()),
                data: data.to_vec(),
                executable,
                owner,
                rent_epoch: 0,
            });
    }

    #[cfg(feature = "anchor")]
    fn add_account_with_anchor<T: AnchorSerialize + Discriminator>(
        &mut self,
        pubkey: Pubkey,
        owner: Pubkey,
        anchor_data: T,
        executable: bool,
    ) {
        let discriminator = &T::discriminator();
        let data = anchor_data.try_to_vec().expect("Cannot serialize provided anchor account");
        let mut v = Vec::new();
        v.extend_from_slice(discriminator);
        v.extend_from_slice(&data); 
        self.add_account_with_data(pubkey, owner, &v, executable);
    }

    fn add_account_with_lamports(
        &mut self,
        pubkey: Pubkey,
        owner: Pubkey,
        lamports: u64,
    ) {
        self.add_account(
            pubkey,
            Account {
                lamports,
                data: vec![],
                executable: false,
                owner,
                rent_epoch: 0,
            }
        );
    }

    fn add_account_with_packable<P: Pack>(
        &mut self,
        pubkey: Pubkey,
        owner: Pubkey,
        data: P,
    ) {
        let data = {
            let mut buf = vec![0u8; P::LEN];
            data.pack_into_slice(&mut buf[..]);
            buf
        };
        self.add_account_with_data(pubkey, owner, &data, false);
    }

    fn add_account_with_borsh<B: BorshSerialize>(&mut self, pubkey: Pubkey, owner: Pubkey, data: B) {
        self.add_account_with_data(
            pubkey,
            owner,
            data.try_to_vec().expect("failed to serialize data").as_ref(),
            false
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
        self.add_account_with_packable(
            pubkey,
            spl_token::ID,
            spl_token::state::Mint {
                mint_authority: COption::from(mint_authority.map(|c| c.clone())),
                supply,
                decimals,
                is_initialized: true,
                freeze_authority: COption::from(freeze_authority.map(|c| c.clone())),
            }
        );
    }

    fn add_token_account(
        &mut self,
        pubkey: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
    ) {
        self.add_account_with_packable(
            pubkey,
            spl_token::ID,
            spl_token::state::Account {
                mint,
                owner,
                amount,
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            }
        );
    }

    fn add_associated_token_account(
        &mut self,
        owner: Pubkey,
        mint: Pubkey,
        amount: u64,
    ) -> Pubkey {
        let pubkey = get_associated_token_address(&owner, &mint);
        self.add_account_with_packable(
            pubkey,
            spl_token::ID,
            spl_token::state::Account {
                mint,
                owner,
                amount,
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            }
        );

        pubkey
    }
}