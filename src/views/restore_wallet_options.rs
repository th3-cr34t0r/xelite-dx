use crate::{
    wallet::utils::{ChatWallet, NETWORK},
    Route, DB, WALLET,
};
use dioxus::{logger::tracing::info, prelude::*};
use sqlx::query;
use tokio::sync::RwLock;

#[component]
pub fn RestoreWalletOptions() -> Element {
    let nav = navigator();

    rsx!(
        button {onclick: move |_| {nav.push(Route::CreateNewWallet {});}, type:"button", "Create New Wallet"},
        button {onclick: move |_| {nav.push(Route::RestoreFromSeed {});}, type:"button", "Restore from Seed"},
        button {onclick: move |_| {nav.push(Route::RestoreFromPrivateKey {});}, type:"button", "Restore from Private Key"},
    )
}

#[component]
pub fn CreateNewWallet() -> Element {
    let nav = navigator();

    let mut wallet_name = use_signal(|| String::new());
    let mut wallet_password = use_signal(|| String::new());
    let mut wallet_msg = use_signal(|| String::new());

    let create_wallet = move |_: FormEvent| async move {
        let name = wallet_name.read().clone();
        let password = wallet_password.read().clone();
        if !name.is_empty() && !password.is_empty() {
            // try to create the requested one
            match ChatWallet::create_wallet(
                name.to_string(),
                password.to_string(),
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
        }
    };

    rsx!(
        div {
            form {
                class: "row",
                onsubmit: create_wallet,

                input {
                    id: "wallet-name-input",
                    placeholder: "Enter a wallet name...",
                    value: "{wallet_name}",
                    oninput: move |event| wallet_name.set(event.value())
                },

                input {
                    id: "wallet-password-input",
                    placeholder: "Enter wallet password...",
                    value: "{wallet_password}",
                    oninput: move |event| wallet_password.set(event.value())
                }
                button { r#type: "submit", "Create Wallet" }
            }
            p { "{wallet_msg.read()}" }
        }
    )
}

#[component]
pub fn RestoreFromSeed() -> Element {
    rsx!(
        div { "Restore From Seed" }
        div { "todo" }
    )
}
#[component]
pub fn RestoreFromPrivateKey() -> Element {
    rsx!(
        div { "Restore From Private Key" }
        div { "todo" }
    )
}
