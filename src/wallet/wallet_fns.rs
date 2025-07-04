use crate::{
    database::db_fns::{db_store_init_message, db_update_status_fee, DbUserLogin},
    views::DbMessage,
    WALLET,
};
use dioxus::{
    hooks::UseFuture,
    logger::tracing::info,
    signals::{Readable, Signal, Writable},
};
use sqlx::{query_as, Error, SqlitePool};
use xelis_common::{config::XELIS_ASSET, utils::format_xelis};

use super::utils::Transfer;

static DEV_FEE_ADDRESS: &str = "xet:gqef8a3qusf476lcqv0f4us947swgf38yrrs3x9npltjzh7mrcrqqgvgex3"; // testnet
pub static DEV_FEE_AMOUNT: f64 = 0.01;

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub async fn wallet_get_seed(
    db: &SqlitePool,
    entered_password: String,
    seed_phrase_info: &mut Signal<String>,
) {
    let db_user: Result<DbUserLogin, Error> = query_as("SELECT username, password FROM user")
        .fetch_one(&*db)
        .await;

    match db_user {
        Ok(db_user) => {
            if entered_password == db_user.password {
                if let Some(wallet) = &*WALLET.read() {
                    match wallet
                        .read()
                        .await
                        .get_mnemonic(crate::wallet::utils::MnemonicLanguage::English)
                        .await
                    {
                        Ok(seed) => {
                            info!("SeedPhrase: {seed}");
                            seed_phrase_info.set(seed);
                        }
                        Err(e) => {
                            info!("SeedPhrase: {e}");
                            seed_phrase_info.set(e.to_string());
                        }
                    }
                } else {
                    info!("Wallet not initialized");
                    seed_phrase_info.set("Wallet not initialized".to_string());
                }
            } else {
                info!("Incorrect password entered");
                seed_phrase_info.set("Incorrect password entered".to_string());
            }
        }
        Err(e) => {
            info!("{e}");
            seed_phrase_info.set(e.to_string());
        }
    }
}

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub async fn wallet_send_message(
    contact_address: String,
    topoheight: i64,
    message: String,
    db_message_handle: &mut UseFuture,
    info: &mut Signal<String>,
) {
    let mut db_message = DbMessage {
        status: "Pending".to_string(),
        direction: "Outgoing".to_string(),
        address: contact_address.clone(),
        hash: Default::default(),
        fee: Default::default(),
        timestamp: Default::default(),
        topoheight,
        asset: XELIS_ASSET.to_string(),
        amount: Default::default(),
        message: Some(message),
    };

    // store it in db
    db_store_init_message(db_message.clone()).await;

    // reload database
    db_message_handle.restart();

    let dev_transfer = Transfer {
        float_amount: DEV_FEE_AMOUNT,
        str_address: DEV_FEE_ADDRESS.to_string(),
        asset_hash: XELIS_ASSET.to_string(),
        extra_data: None,
    };

    let msg_transfer = Transfer {
        float_amount: 0.0,
        str_address: contact_address.clone(),
        asset_hash: XELIS_ASSET.to_string(),
        extra_data: db_message.message.clone(),
    };

    // create the vector of transfers
    let transfers = vec![dev_transfer, msg_transfer];

    // get wallet handle
    match &*WALLET.read() {
        Some(wallet_rw) => {
            let mut wallet = wallet_rw.write().await;

            match wallet.create_transfers_transaction(transfers).await {
                Ok(transaction_summary) => {
                    match wallet
                        .broadcast_transaction(transaction_summary.hash.clone())
                        .await
                    {
                        Ok(_) => {
                            info!("Message sent successfully");

                            wallet.sent_tx_hashes.push(transaction_summary.hash.clone());

                            db_message.status = "Sent".to_string();
                            db_message.hash = transaction_summary.hash;

                            match format_xelis(transaction_summary.fee).parse::<f64>() {
                                Ok(fee) => db_message.fee = fee,
                                Err(e) => info!("Fee parsing error: {e}"),
                            }

                            //update the message in db
                            db_update_status_fee(db_message).await;

                            // reload database
                            db_message_handle.restart();
                        }
                        Err(e) => {
                            info!("{}", e);
                            info.set(format!("error_1: {}", e).to_string());
                        }
                    }
                }
                Err(e) => {
                    info!("{}", e);
                    info.set(format!("error_2: {}", e).to_string());
                }
            };
        }
        None => {
            info!("Error reading wallet");
        }
    }
}
