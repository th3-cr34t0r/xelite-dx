use futures::TryStreamExt;
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
    views::{DbContact, DbMessage},
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
pub async fn db_open_wallet() {
    let nav = navigator();

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
                            nav.push(Route::Home {});
                        }
                        Err(_) => {
                            info!("Wallet couldn't be opened");
                            nav.push(Route::RestoreWalletOptions {});
                        }
                    }
                }
                Err(e) => {
                    info!("Wallet error: {e}");

                    nav.push(Route::RestoreWalletOptions {});
                }
            }
        }
        None => {
            nav.push(Route::RestoreWalletOptions {});
        }
    }
}

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub async fn db_restore_wallet(
    name: String,
    password: String,
    seed: Option<String>,
    private_key: Option<String>,
) {
    let nav = navigator();

    // try to create the requested one
    match ChatWallet::create_wallet(
        name.clone(),
        password.clone(),
        NETWORK,
        seed,
        private_key,
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

                    info!("Wallet created/restored successfully");
                    nav.push(Route::Home {});
                }
                None => {
                    info!("DB openning error");
                }
            }
        }

        Err(e) => {
            info!("Error creating/restoring wallet: {e}");
        }
    }
}

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub async fn db_store_init_message(message: DbMessage) {
    match &*DB.read() {
        Some(db) => {
            // create base table if it does not exist
            query(
                "CREATE TABLE IF NOT EXISTS contacts ( name TEXT NOT NULL, address TEXT NOT NULL )",
            )
            .execute(&*db)
            .await
            .expect("Cannot create contacts DB");

            // get all the contacts
            let all_contacts: Result<Vec<DbContact>, Error> = query_as("SELECT * FROM contacts")
                .fetch(&*db)
                .try_collect()
                .await;

            match all_contacts {
                Ok(all_contacts_vec) => {
                    // check if the address is not contained
                    if all_contacts_vec
                        .iter()
                        .any(|contact| message.address != contact.address)
                    {
                        match query("INSERT INTO contacts (name, address) VALUES (?1, ?2)")
                            .bind(message.address.clone().as_str())
                            .bind(message.address.clone().as_str())
                            .execute(&*db)
                            .await
                        {
                            Ok(_) => {
                                info!("Contact successfully added");
                                db_store_msg(db, message).await;
                            }
                            Err(e) => {
                                info!("Error adding contact to db: {e}");
                                format!("Error adding contact to db: {}", e).to_string();
                            }
                        }
                    } else {
                        info!("Contact already exists, adding the message");
                        db_store_msg(db, message).await;
                    }
                }
                Err(e) => {
                    info!("{e}");
                }
            }
        }
        None => {
            info!("DB read error")
        }
    }
}

#[allow(clippy::await_holding_invalid_type, clippy::borrow_deref_ref)]
async fn db_store_msg(db: &SqlitePool, message: DbMessage) {
    // create Message table if it does not exist
    query(
        "CREATE TABLE IF NOT EXISTS
             Message (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 status TEXT NOT NULL,
                 direction TEXT NOT NULL,
                 address TEXT NOT NULL,
                 hash TEXT,
                 fee REAL,
                 timestamp INTEGER NOT NULL,
                 topoheight INTEGER NOT NULL,
                 asset TEXT NOT NULL,
                 amount INTEGER NOT NULL,
                 message TEXT
             )",
    )
    .execute(&*db)
    .await
    .expect("Cannot create Messages DB");

    // store Message query
    match query(
        "INSERT INTO
             Message (
                 status,
                 direction,
                 address,
                 hash,
                 fee,
                 timestamp,
                 topoheight,
                 asset,
                 amount,
                 message
             ) VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10 )",
    )
    .bind(message.status)
    .bind(message.direction)
    .bind(message.address)
    .bind(message.hash)
    .bind(message.fee)
    .bind(message.timestamp)
    .bind(message.topoheight)
    .bind(message.asset)
    .bind(message.amount)
    .bind(message.message.as_deref())
    .execute(&*db)
    .await
    {
        Ok(_) => info!("Message stored in db"),
        Err(e) => info!("{}", e),
    }
}

#[allow(clippy::await_holding_invalid_type, clippy::borrow_deref_ref)]
pub async fn db_update_status_fee(message: DbMessage) {
    match &*DB.read() {
        Some(db) => {
            // update Message query
            match query("UPDATE Message SET status = ?1, hash = ?2, fee = ?3 WHERE topoheight = ?4 AND message = ?5")
                .bind(message.status)
                .bind(message.hash)
                .bind(message.fee)
                .bind(message.topoheight)
                .bind(message.message)
                .execute(&*db)
                .await
            {
                Ok(_) => info!("Message updated successufully in db"),
                Err(e) => info!("{}", e),
            }
        }

        None => {
            info!("DB read error")
        }
    }
}

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub async fn db_update_status_topoheight(message: DbMessage) {
    match &*DB.read() {
        Some(db) => {
            // update Message query
            match query(
                "UPDATE Message SET status = ?1, topoheight = ?2, timestamp = ?3 WHERE hash = ?4",
            )
            .bind(message.status)
            .bind(message.topoheight)
            .bind(message.timestamp)
            .bind(message.hash)
            .execute(&*db)
            .await
            {
                Ok(_) => info!("Message status and topoheight updated successufully in db"),
                Err(e) => info!("{e}"),
            }
        }

        None => {
            info!("DB read error")
        }
    }
}

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub async fn db_read_messages(
    db: &SqlitePool,
    address: String,
    messages_from_db: &mut Signal<Vec<DbMessage>>,
) {
    let db_messages: Result<Vec<DbMessage>, Error> = query_as(
                 "SELECT status, direction, address, hash, fee, timestamp, topoheight, asset, amount, message FROM Message WHERE address = ?",
             )
             .bind(address)
             .fetch_all(&*db)
             .await;

    match db_messages {
        Ok(db_messages) => {
            // empty the vec
            messages_from_db().clear();

            //assign the new db value
            *messages_from_db.write() = db_messages;
        }
        Err(e) => {
            info!("{}", e);

            // create Message table if it does not exist
            query(
                "CREATE TABLE IF NOT EXISTS
             Message (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 status TEXT NOT NULL,
                 direction TEXT NOT NULL,
                 address TEXT NOT NULL,
                 hash TEXT,
                 fee REAL,
                 timestamp INTEGER NOT NULL,
                 topoheight INTEGER NOT NULL,
                 asset TEXT NOT NULL,
                 amount INTEGER NOT NULL,
                 message TEXT
             )",
            )
            .execute(&*db)
            .await
            .expect("Cannot create Messages DB");
        }
    }
}
#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub async fn db_add_contact(
    db: &SqlitePool,
    contact_ret_msg: &mut Signal<String>,
    new_contact: DbContact,
) {
    // create base table if it does not exist
    query("CREATE TABLE IF NOT EXISTS contacts ( name TEXT NOT NULL, address TEXT NOT NULL )")
        .execute(&*db)
        .await
        .expect("Cannot create contacts DB");

    let all_contacts: Result<Vec<DbContact>, Error> = query_as("SELECT * FROM contacts")
        .fetch(&*db)
        .try_collect()
        .await;

    match all_contacts {
        Ok(all_contacts_vec) => {
            // check if the name is contained
            if !all_contacts_vec
                .iter()
                .any(|contact| contact.name == new_contact.name.clone())
            {
                // check if the address is contained
                if !all_contacts_vec
                    .iter()
                    .any(|contact| contact.address == new_contact.address.clone())
                {
                    match query("INSERT INTO contacts (name, address) VALUES (?1, ?2)")
                        .bind(new_contact.name.as_str())
                        .bind(new_contact.address.as_str())
                        .execute(&*db)
                        .await
                    {
                        Ok(_) => {
                            info!("Contact successfully added");
                            contact_ret_msg.set("Contact successfully added".to_string());
                        }
                        Err(e) => {
                            info!("Error adding contact to db: {e}");
                            contact_ret_msg
                                .set(format!("Error adding contact to db: {}", e).to_string());
                        }
                    }
                } else {
                    info!("Contact address already exists");

                    contact_ret_msg.set("Address already exists".to_string());
                }
            } else {
                info!("Contact name already exists");

                contact_ret_msg.set("Name already exists".to_string());
            }
        }
        Err(e) => {
            info!("{e}");
            contact_ret_msg.set(e.to_string());
        }
    }
}

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub async fn db_remove_contact(db: &SqlitePool, address: String) {
    let nav = navigator();

    match query("DELETE FROM Message WHERE address = ?1")
        .bind(address.clone())
        .execute(&*db)
        .await
    {
        Ok(_) => {
            match query("DELETE FROM contacts WHERE address = ?1")
                .bind(address)
                .execute(&*db)
                .await
            {
                Ok(_) => {
                    info!("Contact removed successfully");
                    // route user to home
                    nav.push(Route::Home {});
                }
                Err(e) => info!("{}", e),
            }
        }
        Err(e) => {
            info!("{}", e);
            match query("DELETE FROM contacts WHERE address = ?1")
                .bind(address)
                .execute(&*db)
                .await
            {
                Ok(_) => {
                    info!("Contact removed successfully");
                    // route user to home
                    nav.push(Route::Home {});
                }
                Err(e) => info!("{}", e),
            }
        }
    }
}

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
pub async fn db_read_contacts(db: &SqlitePool, contacts_vec: &mut Signal<Vec<DbContact>>) {
    // create base table if it does not exist
    query("CREATE TABLE IF NOT EXISTS contacts ( name TEXT NOT NULL, address TEXT NOT NULL )")
        .execute(&*db)
        .await
        .expect("Cannot create contacts DB");

    let db_contacts: Result<Vec<DbContact>, Error> = query_as("SELECT name, address FROM contacts")
        .fetch_all(&*db)
        .await;

    match db_contacts {
        Ok(mut db_contacts) => {
            while let Some(contact_from_db) = db_contacts.pop() {
                info!("{contact_from_db:?}");
                contacts_vec.write().push(DbContact {
                    name: contact_from_db.name,
                    address: contact_from_db.address,
                });
            }
        }
        Err(e) => {
            info!("DbContacts retrived with error {e}");
        }
    }
}
