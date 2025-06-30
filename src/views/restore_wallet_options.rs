use crate::{database::sql::db_add_wallet, Route};
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
            db_add_wallet(name, password);
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
