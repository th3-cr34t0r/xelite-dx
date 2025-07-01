use crate::{database::sql::db_restore_wallet, Route};
use dioxus::prelude::*;

#[component]
pub fn RestoreWalletOptions() -> Element {
    let nav = navigator();

    rsx!(
        button {onclick: move |_| {nav.push(Route::CreateNewWallet {});}, type:"button", "Create New Wallet"},
        button {onclick: move |_| {nav.push(Route::RestoreFromSeed {});}, type:"button", "Restore from Seed"},
        button {onclick: move |_| {nav.push(Route::RestoreFromPrivateKey {});}, type:"button", "Restore from Private Key"},
    )
}

#[allow(clippy::redundant_closure)]
#[component]
pub fn CreateNewWallet() -> Element {
    let mut wallet_name = use_signal(|| String::new());
    let mut wallet_password = use_signal(|| String::new());
    let mut wallet_msg = use_signal(|| String::new());

    let create_wallet = move |_: FormEvent| async move {
        let name = wallet_name.read().clone();
        let password = wallet_password.read().clone();

        // if the name and password are not empty
        if !name.is_empty() && !password.is_empty() {
            // store the info as a wallet login
            db_restore_wallet(name, password, None, None);
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

#[allow(clippy::redundant_closure)]
#[component]
pub fn RestoreFromSeed() -> Element {
    let mut wallet_name = use_signal(|| String::new());
    let mut wallet_password = use_signal(|| String::new());
    let mut wallet_seed = use_signal(|| String::new());

    let restore_wallet = move |_: FormEvent| async move {
        let name = wallet_name.read().clone();
        let password = wallet_password.read().clone();
        let wallet_seed = wallet_seed.read().clone();

        // if the name and password are not empty
        if !name.is_empty() && !password.is_empty() {
            // store the info as a wallet login
            db_restore_wallet(name, password, Some(wallet_seed), None);
        }
    };

    rsx!(
        div {
            form {
                class: "row",
                onsubmit: restore_wallet,

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
                },

                input {
                    id: "wallet-seed-input",
                    placeholder: "Enter wallet seed...",
                    value: "{wallet_seed}",
                    oninput: move |event| wallet_seed.set(event.value())
                }
                button { r#type: "submit", "Restore Wallet" }
            }
        }
    )
}

#[allow(clippy::redundant_closure)]
#[component]
pub fn RestoreFromPrivateKey() -> Element {
    let mut wallet_name = use_signal(|| String::new());
    let mut wallet_password = use_signal(|| String::new());
    let mut wallet_private_key = use_signal(|| String::new());

    let restore_wallet = move |_: FormEvent| async move {
        let name = wallet_name.read().clone();
        let password = wallet_password.read().clone();
        let wallet_private_key = wallet_private_key.read().clone();

        // if the name and password are not empty
        if !name.is_empty() && !password.is_empty() {
            // store the info as a wallet login
            db_restore_wallet(name, password, None, Some(wallet_private_key));
        }
    };

    rsx!(
        div {
            form {
                class: "row",
                onsubmit: restore_wallet,

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
                },

                input {
                    id: "wallet-private-key-input",
                    placeholder: "Enter wallet private key...",
                    value: "{wallet_private_key}",
                    oninput: move |event| wallet_private_key.set(event.value())
                }
                button { r#type: "submit", "Restore Wallet" }
            }
        }
    )
}
