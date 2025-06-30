use std::str::FromStr;

use anyhow::Result;
use dioxus::{logger::tracing::info, prelude::*};
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    query, query_as,
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

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub fn db_open_wallet() {
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
}

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub fn db_add_wallet(name: String, password: String) {
    let wallet_name = use_signal(|| name);
    let wallet_password = use_signal(|| password);

    let nav = navigator();

    use_future(move || async move {
        let name = wallet_name.read().clone();
        let password = wallet_password.read().clone();

        // try to create the requested one
        match ChatWallet::create_wallet(
            name.clone(),
            password.clone(),
            NETWORK,
            None,
            None,
            None,
            None,
        )
        .await
        {
            Ok(wallet) => {
                match &*DB.read() {
                    Some(db) => {
                        // create base table if it does not exist
                        query(
                "CREATE TABLE IF NOT EXISTS user ( username TEXT NOT NULL, password TEXT NOT NULL )",
            )
            .execute(&*db)
            .await
            .expect("Cannot create DB");

                        // store the login info in the database
                        query("INSERT INTO user (username, password) VALUES (?1, ?2)")
                            .bind(name)
                            .bind(password)
                            .execute(&*db)
                            .await
                            .expect("Error storing the login info in the database.");

                        // use the new wallet instance as the app state wallet
                        *WALLET.write() = Some(RwLock::new(wallet));

                        info!("Wallet created successfully");
                        nav.push(Route::Home {});
                    }
                    None => {
                        info!("Db openning error");
                    }
                }
            }

            Err(e) => {
                info!("Error creating wallet: {e}");
            }
        }
    });
}
