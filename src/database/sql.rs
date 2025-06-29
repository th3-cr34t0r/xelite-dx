use std::str::FromStr;

use anyhow::Result;
use dioxus::{logger::tracing::info, prelude::*};
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    query_as,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Error, SqlitePool,
};
use tokio::sync::RwLock;

use crate::{
    wallet::utils::{ChatWallet, NETWORK},
    Route, DB, WALLET,
};

#[derive(Serialize, Deserialize, FromRow)]
pub struct DbUserLogin {
    pub username: String,
    pub password: String,
}

const DB_URS: &str = "sqlite://userdatabase.db";

pub async fn establish_connection() -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            SqliteConnectOptions::from_str(DB_URS)
                .unwrap()
                .create_if_missing(true),
        )
        .await
        .expect("Unable to connect to SQLite database")
}

#[component]
pub fn DbOpenWallet() -> Element {
    let nav = navigator();

    let mut status = use_signal(|| String::new());
    // lead the wallet login from db
    use_future(move || async move {
        // establish db connection
        *DB.write() = Some(establish_connection().await);

        match &*DB.read() {
            Some(db) => {
                // fetch user db entry
                let db_user: Result<DbUserLogin, Error> =
                    query_as("SELECT username, password FROM user")
                        .fetch_one(&*db)
                        .await;

                // in case there is a user in the db, open their wallet
                match db_user {
                    Ok(db_user) => {
                        // try to open the stored wallet
                        match ChatWallet::open_wallet(
                            db_user.username,
                            db_user.password,
                            NETWORK,
                            None,
                            None,
                        )
                        .await
                        {
                            Ok(wallet) => {
                                *WALLET.write() = Some(RwLock::new(wallet));

                                info!("Wallet opened");
                                status.set("Wallet Opened".to_string());
                                nav.push(Route::Home {});
                            }
                            Err(_) => {
                                info!("Wallet couldn't be opened");
                                status.set("Wallet couldn't be opened".to_string());
                                nav.push(Route::RestoreWalletOptions {});
                            }
                        }
                    }
                    Err(e) => {
                        info!("Wallet error: {e}");
                        status.set(format!("Wallet error {}", e).to_string());

                        nav.push(Route::RestoreWalletOptions {});
                    }
                }
            }
            None => {
                status.set("DB NOT CREATED".to_string());
                nav.push(Route::RestoreWalletOptions {});
            }
        }
    });
    rsx!(h1 { "{status.read()}" })
}
