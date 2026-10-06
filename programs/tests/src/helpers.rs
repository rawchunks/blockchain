use {
    litesvm::{types::FailedTransactionMetadata, LiteSVM},
    rawchunks_common::{
        SEED_CONSUMER, SEED_FACTORY, SEED_LICENSE, SEED_REMIX, SEED_REMIX_AUTHORITY,
        SEED_REMIX_COUNT, SEED_ROYALTIES, SEED_SONG,
    },
    solana_account::Account,
    solana_address::{address, Address},
    solana_instruction::Instruction,
    solana_keypair::Keypair,
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_token_interface::{
        state::{Account as TokenAccount, AccountState, Mint},
        ID as TOKEN_PROGRAM_ID,
    },
    std::path::PathBuf,
};

pub const FACTORY_PROGRAM_ID: Address = address!("EPKxVRtMaZbnoPgtpycYhXX7EGM3PzyCM36upmwJyenD");
pub const ROYALTIES_PROGRAM_ID: Address = address!("Gs7HCcMXgqfdQFyGdfggEsNTASiobu1T69kZJUXcRtWv");
pub const REMIX_PROGRAM_ID: Address = address!("5miVCjgcGe85EKRjTNSc8rbcd8pF64GUDHPNtFqzEtHr");

pub fn deploy_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/deploy")
        .join(format!("{}.so", name))
}

pub fn to_pubkey(addr: &Address) -> Pubkey {
    Pubkey::new_from_array(addr.to_bytes())
}

pub fn find_royalties_config_pda(program_id: &Address) -> (Address, u8) {
    Address::find_program_address(&[SEED_ROYALTIES], program_id)
}

pub fn find_factory_config_pda(program_id: &Address) -> (Address, u8) {
    Address::find_program_address(&[SEED_FACTORY], program_id)
}

pub fn find_song_pda(song_id: u64, program_id: &Address) -> (Address, u8) {
    Address::find_program_address(&[SEED_SONG, &song_id.to_le_bytes()], program_id)
}

pub fn find_authorized_consumer_pda(consumer: &Address, program_id: &Address) -> (Address, u8) {
    Address::find_program_address(&[SEED_CONSUMER, consumer.as_ref()], program_id)
}

pub fn find_license_record_pda(song_id: u64, user: &Address, program_id: &Address) -> (Address, u8) {
    Address::find_program_address(&[SEED_LICENSE, &song_id.to_le_bytes(), user.as_ref()], program_id)
}

pub fn find_remix_config_pda(program_id: &Address) -> (Address, u8) {
    Address::find_program_address(&[SEED_REMIX], program_id)
}

pub fn find_remix_authority_pda(program_id: &Address) -> (Address, u8) {
    Address::find_program_address(&[SEED_REMIX_AUTHORITY], program_id)
}

pub fn find_song_remix_count_pda(original_song_id: u64, program_id: &Address) -> (Address, u8) {
    Address::find_program_address(&[SEED_REMIX_COUNT, &original_song_id.to_le_bytes()], program_id)
}

pub fn find_remix_record_pda(original_song_id: u64, remix_index: u64, program_id: &Address) -> (Address, u8) {
    Address::find_program_address(
        &[SEED_REMIX, &original_song_id.to_le_bytes(), &remix_index.to_le_bytes()],
        program_id,
    )
}

pub struct TestEnv {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub admin: Keypair,
}

impl TestEnv {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new().with_default_programs();

        let factory_bytes = std::fs::read(deploy_path("rawchunks_factory")).expect("factory.so");
        let royalties_bytes = std::fs::read(deploy_path("rawchunks_royalties")).expect("royalties.so");
        let remix_bytes = std::fs::read(deploy_path("rawchunks_remix")).expect("remix.so");

        svm.add_program(FACTORY_PROGRAM_ID, &factory_bytes).unwrap();
        svm.add_program(ROYALTIES_PROGRAM_ID, &royalties_bytes).unwrap();
        svm.add_program(REMIX_PROGRAM_ID, &remix_bytes).unwrap();

        let payer = Keypair::new();
        let admin = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
        svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();

        Self { svm, payer, admin }
    }

    pub fn admin(&self) -> Keypair {
        self.admin.insecure_clone()
    }

    pub fn payer(&self) -> Keypair {
        self.payer.insecure_clone()
    }

    pub fn airdrop(&mut self, target: &Address, lamports: u64) {
        self.svm.airdrop(target, lamports).unwrap();
    }

    pub fn setup_mint(&mut self, mint_authority: &Address) -> Address {
        let mint_keypair = Keypair::new();
        let mint_address = mint_keypair.pubkey();
        let mint_state = Mint {
            mint_authority: COption::Some(to_pubkey(mint_authority)),
            supply: 1_000_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        };
        let mut data = [0u8; Mint::LEN];
        mint_state.pack_into_slice(&mut data);

        self.svm
            .set_account(
                mint_address,
                Account {
                    lamports: 100_000_000,
                    data: data.to_vec(),
                    owner: TOKEN_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

        mint_address
    }

    pub fn setup_token_account(&mut self, owner: &Address, mint: &Address, initial_amount: u64) -> Address {
        let account_keypair = Keypair::new();
        let account_address = account_keypair.pubkey();
        let token_state = TokenAccount {
            mint: to_pubkey(mint),
            owner: to_pubkey(owner),
            amount: initial_amount,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        };
        let mut data = [0u8; TokenAccount::LEN];
        token_state.pack_into_slice(&mut data);

        self.svm
            .set_account(
                account_address,
                Account {
                    lamports: 100_000_000,
                    data: data.to_vec(),
                    owner: TOKEN_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

        account_address
    }

    pub fn get_token_balance(&self, token_account: &Address) -> u64 {
        let account = self.svm.get_account(token_account).expect("token account exists");
        let parsed = TokenAccount::unpack_from_slice(&account.data).expect("valid token account");
        parsed.amount
    }

    pub fn send_tx(&mut self, ix: Instruction, signers: &[&Keypair]) -> Result<u64, FailedTransactionMetadata> {
        let blockhash = self.svm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&signers[0].pubkey()),
            signers,
            blockhash,
        );
        let res = self.svm.send_transaction(tx)?;
        Ok(res.compute_units_consumed)
    }

    pub fn send_tx_multi_ix(
        &mut self,
        ixs: &[Instruction],
        signers: &[&Keypair],
    ) -> Result<u64, FailedTransactionMetadata> {
        let blockhash = self.svm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(
            ixs,
            Some(&signers[0].pubkey()),
            signers,
            blockhash,
        );
        let res = self.svm.send_transaction(tx)?;
        Ok(res.compute_units_consumed)
    }
}
