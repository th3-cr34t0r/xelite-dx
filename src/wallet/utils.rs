use anyhow::{anyhow, bail, Context, Result};
use dioxus::logger::tracing::info;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use xelis_common::{
    api::{
        wallet::{EntryType, TransactionEntry},
        DataElement, DataValue,
    },
    config::{COIN_DECIMALS, XELIS_ASSET},
    crypto::{Address, Hash, Hashable},
    serializer::Serializer,
    transaction::{
        builder::{FeeBuilder, TransactionTypeBuilder, TransferBuilder},
        Transaction,
    },
    utils::{format_coin, format_xelis},
};
use xelis_wallet::{
    config::LogProgressTableGenerationReportFunction,
    entry::EntryData,
    precomputed_tables::{self, L1_FULL, L1_LOW, L1_MEDIUM},
    transaction_builder::TransactionBuilderState,
    wallet::{Event, RecoverOption, Wallet},
};

pub use xelis_common::network::Network;

use crate::views::DbMessage;

// pub const NODE_ENDPOINT: &str = "node.xelis.io"; //mainnet
// pub static NETWORK: Network = Network::Mainnet; //mainnet

pub const NODE_ENDPOINT: &str = "testnet-node.xelis.io"; //testnet
pub static NETWORK: Network = Network::Testnet; //testnet

// pub const NODE_ENDPOINT: &str = "192.168.100.7:8080"; // local

#[derive(Clone, Debug)]
pub struct Transfer {
    pub float_amount: f64,
    pub str_address: String,
    pub asset_hash: String,
    pub extra_data: Option<String>,
}
// ECDLP Tables L1 size
pub enum TableSize {
    L1Low,
    L1Medium,
    L1Full,
}

impl TableSize {
    pub fn convert(&self) -> usize {
        match self {
            TableSize::L1Low => L1_LOW,
            TableSize::L1Medium => L1_MEDIUM,
            TableSize::L1Full => L1_FULL,
        }
    }
}

pub enum MnemonicLanguage {
    English,
    French,
    Italian,
    Spanish,
    Portuguese,
    Japanese,
    ChineseSimplified,
    Russian,
    Esperanto,
    Dutch,
    German,
}

impl MnemonicLanguage {
    pub fn convert(&self) -> usize {
        match self {
            MnemonicLanguage::English => 0,
            MnemonicLanguage::French => 1,
            MnemonicLanguage::Italian => 2,
            MnemonicLanguage::Spanish => 3,
            MnemonicLanguage::Portuguese => 4,
            MnemonicLanguage::Japanese => 5,
            MnemonicLanguage::ChineseSimplified => 6,
            MnemonicLanguage::Russian => 7,
            MnemonicLanguage::Esperanto => 8,
            MnemonicLanguage::Dutch => 9,
            MnemonicLanguage::German => 10,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SummaryTransaction {
    pub hash: String,
    fee: u64,
    transaction_type: TransactionTypeBuilder,
}

pub struct ChatWallet {
    wallet: Arc<Wallet>,
    pub rx_messages: Vec<DbMessage>,
    pub balance: String,
    pub topoheight: i64,
    pub is_online: bool,
    // pub income_tx: Vec<IncomeTx>,
    pub pending_transactions: Arc<RwLock<HashMap<Hash, (Transaction, TransactionBuilderState)>>>,
}

// static CACHED_TABLES: Mutex<Option<precomputed_tables::PrecomputedTablesShared>> = Mutex::new(None);s

impl ChatWallet {
    /// Create or restore wallet
    pub async fn create_wallet(
        name: String,
        password: String,
        network: Network,
        seed: Option<String>,
        private_key: Option<String>,
        precomputed_tables_path: Option<String>,
        precomputed_table_size: Option<TableSize>,
    ) -> Result<ChatWallet> {
        // recover wallet by seed or private_key
        let seed: Option<RecoverOption> = if let Some(seed) = seed.as_deref() {
            Some(RecoverOption::Seed(seed))
        } else {
            private_key.as_deref().map(RecoverOption::PrivateKey)
        };

        // get the size conversion or default
        let precomputed_table_size = if let Some(precomputed_tables_size) = precomputed_table_size {
            precomputed_tables_size.convert()
        } else {
            L1_LOW
        };

        // creates a new table
        let precomputed_tables = precomputed_tables::read_or_generate_precomputed_tables(
            precomputed_tables_path.as_deref(),
            precomputed_table_size,
            LogProgressTableGenerationReportFunction,
            true,
        )
        .await?;

        let n_threads = 4; // hardcoding to 4 for debug purposes

        // create a wallet instance
        let chat_wallet = Wallet::create(
            &name,
            &password,
            seed,
            network,
            precomputed_tables,
            n_threads,
            n_threads,
        )
        .await?;

        Ok(ChatWallet {
            wallet: chat_wallet,
            rx_messages: Vec::new(),
            balance: String::new(),
            topoheight: 0,
            is_online: false,
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Open existing wallet
    pub async fn open_wallet(
        name: String,
        password: String,
        network: Network,
        precomputed_tables_path: Option<String>,
        precomputed_table_size: Option<TableSize>,
    ) -> Result<ChatWallet> {
        // get the size conversion or default
        let precomputed_table_size = if let Some(precomputed_tables_size) = precomputed_table_size {
            precomputed_tables_size.convert()
        } else {
            L1_LOW
        };

        // creates or restore a table
        let precomputed_tables = precomputed_tables::read_or_generate_precomputed_tables(
            precomputed_tables_path.as_deref(),
            precomputed_table_size,
            LogProgressTableGenerationReportFunction,
            true,
        )
        .await?;

        let n_threads = 4; // hardcoding rn

        let chat_wallet = Wallet::open(
            name.as_str(),
            password.as_str(),
            network,
            precomputed_tables,
            n_threads,
            n_threads,
        )?;

        Ok(ChatWallet {
            wallet: chat_wallet,
            rx_messages: Vec::new(),
            balance: String::new(),
            topoheight: 0,
            is_online: false,
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    async fn process_incoming_tx(&self, transaction: TransactionEntry) -> DbMessage {
        let mut rx_message = DbMessage {
            direction: "Incoming".to_string(),
            address: Default::default(),
            hash: transaction.hash.to_string(),
            timestamp: transaction.timestamp as i64,
            topoheight: transaction.topoheight as i64,
            asset: Default::default(),
            amount: Default::default(),
            message: Default::default(),
        };

        let entry_data = transaction.entry;

        if let EntryType::Incoming { from, transfers } = entry_data {
            rx_message.address = from.as_string().unwrap();
            let transfer = transfers[0].clone();

            rx_message.asset = transfer.asset.to_string();
            rx_message.amount = transfer.amount as i64;

            rx_message.message = match transfer.extra_data {
                Some(extra_data) => extra_data
                    .data()
                    .map(|extra_data| extra_data.clone().to_value().unwrap().to_string().unwrap()),
                None => None,
            };
        }

        info!("Transformed Transaction: {rx_message:#?}");
        rx_message
    }

    pub async fn backgroud_daemon(&mut self) {
        let wallet = self.get_wallet().await;
        let mut receiver = wallet.subscribe_events().await;

        if let Ok(event) = receiver.recv().await {
            match event {
                Event::NewTransaction(transaction) => {
                    info!("NewTransaction");

                    let transaction = self.process_incoming_tx(transaction).await;

                    // only store tranactions with messages
                    if transaction.message.is_some() {
                        self.rx_messages.push(transaction);
                    } else {
                        // only balance has changed
                    }

                    info!("WalletTransactions: {:#?}", self.rx_messages);
                }
                Event::NewTopoHeight { topoheight } => {
                    info!("NewTopoHeight: {topoheight}");
                    self.topoheight = topoheight as i64;
                }
                Event::BalanceChanged(new_balance) => {
                    info!("BalanceChanged: {new_balance:?}");
                    if new_balance.asset == XELIS_ASSET {
                        self.balance = format_xelis(new_balance.balance);
                    }
                }
                _ => {}
            }
        }
    }

    /// Get wallet txs
    pub async fn get_rx_extra_data(
        &self,
        min_topoheight: u64,
        contact_address: String,
    ) -> Result<Vec<String>> {
        // read wallet storage
        let storage = self.wallet.get_storage();

        // get public key
        let wallet_public_key = self.wallet.get_public_key();

        let mut data_vec: Vec<String> = vec![String::new()];

        // load wallet txs
        if let Ok(txs) = storage.read().await.get_filtered_transactions(
            Some(wallet_public_key),
            Some(&XELIS_ASSET),
            Some(min_topoheight),
            None,
            true,
            false,
            false,
            false,
            None,
            None,
            None,
        ) {
            for tx in txs.iter() {
                if let EntryData::Incoming { from, transfers } = tx.get_entry().clone() {
                    if from.to_address(false).to_string() == contact_address {
                        for transfer in transfers {
                            if let Some(extra_data) = transfer.get_extra_data() {
                                if let Some(DataElement::Value(data_element)) = extra_data.data() {
                                    if let Ok(data) = data_element.as_string() {
                                        data_vec.push(data.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(data_vec)
        } else {
            Ok(vec!["Cannot get wallet txs".to_string()])
        }
    }

    /// Change wallet password
    pub async fn change_password(&self, old_password: String, new_password: String) -> Result<()> {
        self.wallet.set_password(&old_password, &new_password).await
    }

    /// Set the wallet to online mode
    pub async fn set_online(&self, daemon_address: String) -> Result<()> {
        Ok(self.wallet.set_online_mode(&daemon_address, true).await?)
    }

    // Set the wallet to offline mode
    pub async fn set_offline(&self) -> Result<()> {
        self.wallet.set_offline_mode().await?;

        Ok(())
    }

    /// Check if wallet is online
    pub async fn is_online(&self) -> bool {
        self.wallet.is_online().await
    }

    /// Get the wallet address as String
    pub async fn get_address(&self) -> String {
        self.wallet.get_address().to_string()
    }

    /// Get wallet access
    pub async fn get_wallet(&self) -> &Arc<Wallet> {
        &self.wallet // returns a reference to private field wallet
    }

    /// Close the wallet
    pub async fn close_wallet(&self) -> Result<()> {
        self.wallet.close().await;
        Ok(())
    }

    /// Get the wallet mnemonic phrase
    pub async fn get_mnemonic(&self, language: MnemonicLanguage) -> Result<String> {
        self.wallet.get_seed(language.convert())
    }

    /// Get the wallet nonce
    pub async fn get_nonce(&self) -> u64 {
        self.wallet.get_nonce().await
    }

    /// Checks if the provided password is valid
    pub async fn is_valid_password(&self, password: String) -> Result<()> {
        self.wallet.is_valid_password(&password).await
    }

    /// Gets the Xelis balance
    pub async fn get_balance(&mut self) -> Result<String> {
        let storage = self.wallet.get_storage().read().await;
        let balance = storage.get_plaintext_balance_for(&XELIS_ASSET).await?;
        let formatted_balance = format_xelis(balance);
        self.balance = formatted_balance.clone();

        Ok(formatted_balance)
    }

    /// Format atomic units to human readable format
    pub async fn format_coin(
        &self,
        atomic_amount: u64,
        asset_hash: Option<String>,
    ) -> Result<String> {
        let asset = match asset_hash {
            Some(asset) => Hash::from_hex(&asset).context("Invalid Asset")?,
            None => XELIS_ASSET,
        };

        let decimals = {
            let storage = self.wallet.get_storage().read().await;
            let asset = storage
                .get_asset(&asset)
                .await
                .context("Asset not found in storage")?;

            asset.get_decimals()
        };

        Ok(format_coin(atomic_amount, decimals))
    }

    /// Estimates the fees for a transaction
    pub async fn estimate_fees(&self, transfers: Vec<Transfer>) -> Result<String> {
        let transaction_builder = self
            .create_transfers(transfers)
            .await
            .context("Error while creating transaction type builder")?;

        let estimated_fees = self
            .wallet
            .estimate_fees(transaction_builder, FeeBuilder::default())
            .await
            .context("Error while estimating fees")?;

        Ok(format_coin(estimated_fees, COIN_DECIMALS))
    }

    /// Creates transfer transaction
    pub async fn create_transfers_transaction(
        &mut self,
        transfers: Vec<Transfer>,
    ) -> Result<SummaryTransaction> {
        // clear the pending transacitons
        self.pending_transactions.write().unwrap().clear();

        info!("Building Transaction...");

        let transaction_type_builder = self
            .create_transfers(transfers)
            .await
            .context("Error while creating transaction type builder")?;

        let (tx, state) = {
            let storage = self.wallet.get_storage().write().await;
            self.wallet
                .create_transaction_with_storage(
                    &storage,
                    transaction_type_builder.clone(),
                    FeeBuilder::default(),
                )
                .await?
        };

        let tx_hash = tx.hash();
        let tx_fee = tx.get_fee();
        info!("Transaction created!");
        info!("TX Hash: {}", tx_hash);
        info!("TX Fee: {}", tx_fee);

        self.pending_transactions
            .write()
            .unwrap()
            .insert(tx_hash.clone(), (tx, state));

        Ok(SummaryTransaction {
            hash: tx_hash.to_hex(),
            fee: tx_fee,
            transaction_type: transaction_type_builder,
        })
    }

    /// Get daemon info
    pub async fn get_daemon_info(&self) -> Result<String> {
        let network_handler = self.wallet.get_network_handler().lock().await;

        if let Some(handler) = network_handler.as_ref() {
            let api = handler.get_api();

            let info = api.get_info().await?;

            Ok(serde_json::to_string(&info)?)
        } else {
            Err(anyhow!("Network handler not available"))
        }
    }

    /// Clears a transaction
    pub async fn clear_transaction(
        &self,
        tx_hash: String,
    ) -> Result<(Transaction, TransactionBuilderState)> {
        let tx_hash = Hash::from_hex(&tx_hash)?;

        let result = self
            .pending_transactions
            .write()
            .unwrap()
            .remove(&tx_hash)
            .context("Cannot remove the pending transaction");

        info!("Tx: {tx_hash} removed from pending transactions!");

        result
    }

    /// Broadcasts a transaction to the network
    pub async fn broadcast_transaction(&self, tx_hash: String) -> Result<()> {
        info!("start to broadcast tx: {}", tx_hash);

        if self.wallet.is_online().await {
            let (tx, mut state) = self.clear_transaction(tx_hash.clone()).await?;
            let mut storage = self.wallet.get_storage().write().await;

            info!("Broadcasting transaction...");
            if let Err(e) = self.wallet.submit_transaction(&tx).await {
                error!("Error while submitting transaction, clearing cache...");
                storage.clear_tx_cache();
                storage.delete_unconfirmed_balances().await;

                warn!("Inserting back to pending transactions in case of retry...");
                let hash: Hash = Hash::from_hex(&tx_hash)?;
                self.pending_transactions
                    .write()
                    .unwrap()
                    .insert(hash, (tx, state));

                bail!(e)
            } else {
                info!("Transaction submitted successfully!");
                state.apply_changes(&mut storage).await?;
                info!("Transaction applied to storage");
            }
        } else {
            return Err(anyhow!(
                "Wallet is offline, transaction cannot be submitted"
            ));
        }

        Ok(())
    }

    /// Private method for creating TransactionTypeBuilder from transfers
    async fn create_transfers(&self, transfers: Vec<Transfer>) -> Result<TransactionTypeBuilder> {
        let mut vec = Vec::new();

        for transfer in transfers {
            let asset = Hash::from_hex(&transfer.asset_hash).context("Invalid asset")?;

            let amount = self
                .convert_float_amount(transfer.float_amount, &asset)
                .await?;

            let address = Address::from_string(&transfer.str_address).context("Invalid address")?;

            let extra_data = transfer
                .extra_data
                .map(|data| DataElement::Value(DataValue::String(data)));

            let transfer_builder = TransferBuilder {
                destination: address,
                amount,
                asset,
                extra_data: extra_data.clone(),
                encrypt_extra_data: extra_data.is_some(),
            };

            vec.push(transfer_builder);
        }

        Ok(TransactionTypeBuilder::Transfers(vec))
    }

    /// Private method that converts float amout to atomic format
    async fn convert_float_amount(&self, float_amount: f64, asset: &Hash) -> Result<u64> {
        let storage = self.wallet.get_storage().read().await;
        let decimals = storage.get_asset(asset).await?.get_decimals();
        let amount = (float_amount * 10u32.pow(decimals as u32) as f64) as u64;
        Ok(amount)
    }
}
