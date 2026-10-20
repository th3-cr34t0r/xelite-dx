use crate::{database::db_fns::db_restore_wallet, Route};
use dioxus::prelude::*;

#[component]
pub fn RestoreWalletOptions() -> Element {
    let nav = navigator();

    rsx!(
        div {
            style: "display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:100vh; background-color:#0a0a0a; color:#ffffff; padding:24px;",
            h1 {
                style: "font-size:32px; font-weight:bold; margin-bottom:8px;",
                "Xelite DX"
            }
            p { style: "color:#9ca3af; margin-bottom:40px;", "Wallet Setup" }

            button {
                style: "width:100%; max-width:24rem; padding:12px 24px; background-color:#2563eb; color:white; font-size:18px; font-weight:500; border-radius:8px; border:none; cursor:pointer;",
                onclick: move |_| { nav.push(Route::CreateNewWallet {}); },
                type: "button",
                "+ Create New Wallet"
            }

            button {
                style: "width:100%; max-width:24rem; padding:12px 24px; background-color:#1f2937; color:white; font-size:18px; font-weight:500; border-radius:8px; border:none; cursor:pointer; margin-top:16px;",
                onclick: move |_| { nav.push(Route::RestoreFromSeed {}); },
                type: "button",
                "🌱 Restore from Seed Phrase"
            }

            button {
                style: "width:100%; max-width:24rem; padding:12px 24px; background-color:#1f2937; color:white; font-size:18px; font-weight:500; border-radius:8px; border:none; cursor:pointer; margin-top:16px;",
                onclick: move |_| { nav.push(Route::RestoreFromPrivateKey {}); },
                type: "button",
                "🔑 Restore from Private Key"
            }
        }
    )
}

#[component]
pub fn CreateNewWallet() -> Element {
    let mut wallet_name = use_signal(|| String::new());
    let mut wallet_password = use_signal(|| String::new());
    let mut wallet_msg = use_signal(|| String::new());
    let nav = navigator();

    let create_wallet = move |_: FormEvent| async move {
        let name = wallet_name.read().clone();
        let password = wallet_password.read().clone();

        if !name.is_empty() && !password.is_empty() {
            wallet_msg.set("Creating wallet...".to_string());
            db_restore_wallet(name, password, None, None).await;
        } else {
            wallet_msg.set("Please fill in all fields".to_string());
        }
    };

    rsx!(
        div {
            style: "display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:100vh; background-color:#0a0a0a; color:#ffffff; padding:24px;",

            button {
                style: "color:#9ca3af; margin-bottom:24px; background:none; border:none; cursor:pointer; font-size:14px; align-self:flex-start; max-width:24rem; width:100%;",
                onclick: move |_| { nav.push(Route::RestoreWalletOptions {}); },
                type: "button",
                "← Back"
            }

            h1 { style: "font-size:28px; font-weight:bold; margin-bottom:32px;", "Create New Wallet" }

            form {
                style: "display:flex; flex-direction:column; gap:16px; width:100%; max-width:24rem; background-color:#111827; padding:24px; border-radius:12px;",
                onsubmit: create_wallet,

                label { style: "display:flex; flex-direction:column; gap:4px; font-size:14px; color:#d1d5db;",
                    "Wallet Name"
                    input {
                        style: "margin-top:4px; padding:8px 16px; background-color:#1f2937; border:1px solid #374151; border-radius:8px; color:white; outline:none;",
                        placeholder: "Enter a wallet name...",
                        value: "{wallet_name}",
                        oninput: move |event| wallet_name.set(event.value())
                    }
                }

                label { style: "display:flex; flex-direction:column; gap:4px; font-size:14px; color:#d1d5db;",
                    "Password"
                    input {
                        style: "margin-top:4px; padding:8px 16px; background-color:#1f2937; border:1px solid #374151; border-radius:8px; color:white; outline:none;",
                        r#type: "password",
                        placeholder: "Enter wallet password...",
                        value: "{wallet_password}",
                        oninput: move |event| wallet_password.set(event.value())
                    }
                }

                button {
                    style: "padding:12px; background-color:#2563eb; color:white; font-size:18px; font-weight:500; border-radius:8px; border:none; cursor:pointer;",
                    r#type: "submit",
                    "Create Wallet"
                }
            }

            if !wallet_msg.read().is_empty() {
                p { style: "margin-top:16px; color:#d1d5db; text-align:center;", "{wallet_msg.read()}" }
            }
        }
    )
}

#[component]
pub fn RestoreFromSeed() -> Element {
    let mut wallet_name = use_signal(|| String::new());
    let mut wallet_password = use_signal(|| String::new());
    let mut wallet_seed = use_signal(|| String::new());
    let nav = navigator();

    let restore_wallet = move |_: FormEvent| async move {
        let name = wallet_name.read().clone();
        let password = wallet_password.read().clone();
        let seed = wallet_seed.read().clone();

        if !name.is_empty() && !password.is_empty() && !seed.is_empty() {
            db_restore_wallet(name, password, Some(seed), None).await;
        } else {
            info!("Please fill in all fields");
        }
    };

    rsx!(
        div {
            style: "display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:100vh; background-color:#0a0a0a; color:#ffffff; padding:24px;",

            button {
                style: "color:#9ca3af; margin-bottom:24px; background:none; border:none; cursor:pointer; font-size:14px; align-self:flex-start; max-width:24rem; width:100%;",
                onclick: move |_| { nav.push(Route::RestoreWalletOptions {}); },
                type: "button",
                "← Back"
            }

            h1 { style: "font-size:28px; font-weight:bold; margin-bottom:8px;", "Restore from Seed Phrase" }
            p { style: "color:#9ca3af; font-size:14px; margin-bottom:32px;", "Enter your 12 or 24-word seed phrase" }

            form {
                style: "display:flex; flex-direction:column; gap:16px; width:100%; max-width:24rem; background-color:#111827; padding:24px; border-radius:12px;",
                onsubmit: restore_wallet,

                label { style: "display:flex; flex-direction:column; gap:4px; font-size:14px; color:#d1d5db;",
                    "Wallet Name"
                    input {
                        style: "margin-top:4px; padding:8px 16px; background-color:#1f2937; border:1px solid #374151; border-radius:8px; color:white; outline:none;",
                        placeholder: "Enter a wallet name...",
                        value: "{wallet_name}",
                        oninput: move |event| wallet_name.set(event.value())
                    }
                }

                label { style: "display:flex; flex-direction:column; gap:4px; font-size:14px; color:#d1d5db;",
                    "Password"
                    input {
                        style: "margin-top:4px; padding:8px 16px; background-color:#1f2937; border:1px solid #374151; border-radius:8px; color:white; outline:none;",
                        r#type: "password",
                        placeholder: "Enter wallet password...",
                        value: "{wallet_password}",
                        oninput: move |event| wallet_password.set(event.value())
                    }
                }

                label { style: "display:flex; flex-direction:column; gap:4px; font-size:14px; color:#d1d5db;",
                    "Seed Phrase"
                    textarea {
                        style: "margin-top:4px; padding:8px 16px; background-color:#1f2937; border:1px solid #374151; border-radius:8px; color:white; outline:none; resize:vertical;",
                        rows: 3,
                        placeholder: "word1 word2 word3 ...",
                        value: "{wallet_seed}",
                        oninput: move |event| wallet_seed.set(event.value())
                    }
                }

                button {
                    style: "padding:12px; background-color:#2563eb; color:white; font-size:18px; font-weight:500; border-radius:8px; border:none; cursor:pointer;",
                    r#type: "submit",
                    "Restore Wallet"
                }
            }
        }
    )
}

#[component]
pub fn RestoreFromPrivateKey() -> Element {
    let mut wallet_name = use_signal(|| String::new());
    let mut wallet_password = use_signal(|| String::new());
    let mut wallet_private_key = use_signal(|| String::new());
    let nav = navigator();

    let restore_wallet = move |_: FormEvent| async move {
        let name = wallet_name.read().clone();
        let password = wallet_password.read().clone();
        let key = wallet_private_key.read().clone();

        if !name.is_empty() && !password.is_empty() && !key.is_empty() {
            db_restore_wallet(name, password, None, Some(key)).await;
        } else {
            info!("Please fill in all fields");
        }
    };

    rsx!(
        div {
            style: "display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:100vh; background-color:#0a0a0a; color:#ffffff; padding:24px;",

            button {
                style: "color:#9ca3af; margin-bottom:24px; background:none; border:none; cursor:pointer; font-size:14px; align-self:flex-start; max-width:24rem; width:100%;",
                onclick: move |_| { nav.push(Route::RestoreWalletOptions {}); },
                type: "button",
                "← Back"
            }

            h1 { style: "font-size:28px; font-weight:bold; margin-bottom:8px;", "Restore from Private Key" }
            p { style: "color:#9ca3af; font-size:14px; margin-bottom:32px;", "⚠️ Keep your private key secret" }

            form {
                style: "display:flex; flex-direction:column; gap:16px; width:100%; max-width:24rem; background-color:#111827; padding:24px; border-radius:12px;",
                onsubmit: restore_wallet,

                label { style: "display:flex; flex-direction:column; gap:4px; font-size:14px; color:#d1d5db;",
                    "Wallet Name"
                    input {
                        style: "margin-top:4px; padding:8px 16px; background-color:#1f2937; border:1px solid #374151; border-radius:8px; color:white; outline:none;",
                        placeholder: "Enter a wallet name...",
                        value: "{wallet_name}",
                        oninput: move |event| wallet_name.set(event.value())
                    }
                }

                label { style: "display:flex; flex-direction:column; gap:4px; font-size:14px; color:#d1d5db;",
                    "Password"
                    input {
                        style: "margin-top:4px; padding:8px 16px; background-color:#1f2937; border:1px solid #374151; border-radius:8px; color:white; outline:none;",
                        r#type: "password",
                        placeholder: "Enter wallet password...",
                        value: "{wallet_password}",
                        oninput: move |event| wallet_password.set(event.value())
                    }
                }

                label { style: "display:flex; flex-direction:column; gap:4px; font-size:14px; color:#d1d5db;",
                    "Private Key"
                    input {
                        style: "margin-top:4px; padding:8px 16px; background-color:#1f2937; border:1px solid #374151; border-radius:8px; color:white; outline:none;",
                        r#type: "password",
                        placeholder: "Enter your private key...",
                        value: "{wallet_private_key}",
                        oninput: move |event| wallet_private_key.set(event.value())
                    }
                }

                button {
                    style: "padding:12px; background-color:#2563eb; color:white; font-size:18px; font-weight:500; border-radius:8px; border:none; cursor:pointer;",
                    r#type: "submit",
                    "Restore Wallet"
                }
            }
        }
    )
}
