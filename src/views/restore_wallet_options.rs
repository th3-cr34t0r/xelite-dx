use crate::{database::db_fns::db_restore_wallet, Route};
use crate::views::theme::{
    grid_overlay, hex_pattern, BG, DIM, HAIR, MINT, MONO, PANEL, TEXT,
};
use dioxus::{logger::tracing::info, prelude::*};

fn back_button_style() -> String {
    format!(
        "font-size:12px; color:{DIM}; background:none; border:1px solid {HAIR}; border-radius:6px; padding:8px 16px; cursor:pointer; font-family:{MONO}; letter-spacing:2px;"
    )
}

fn field_style() -> String {
    format!(
        "width:100%; box-sizing:border-box; padding:12px 16px; background-color:{BG}; border:1px solid {HAIR}; border-radius:6px; color:{TEXT}; outline:none; font-size:14px; font-family:{MONO};"
    )
}

fn field_label_style() -> String {
    format!("font-size:9px; color:{DIM}; letter-spacing:2px; margin-bottom:8px;")
}

fn submit_style() -> String {
    format!(
        "padding:14px 24px; background:none; color:{MINT}; font-size:13px; font-weight:bold; border-radius:6px; border:1px solid {MINT}; cursor:pointer; font-family:{MONO}; letter-spacing:3px; box-shadow:0 0 12px rgba(100,255,218,0.12); width:100%;"
    )
}

fn card_style() -> String {
    format!(
        "display:flex; flex-direction:column; gap:20px; width:100%; max-width:360px; background-color:{PANEL}; padding:28px; border-radius:8px; border:1px solid {HAIR}; position:relative; z-index:1;"
    )
}

fn glow_title(size: u32, text: &'static str) -> Element {
    rsx! {
        h1 {
            style: format!("font-size:{size}px; font-weight:bold; margin:0 0 10px 0; color:{MINT}; font-family:{MONO}; text-shadow:0 0 16px rgba(100,255,218,0.35); letter-spacing:4px; text-align:center;"),
            "{text}"
        }
    }
}

#[component]
pub fn RestoreWalletOptions() -> Element {
    let nav = navigator();

    rsx! {
        div {
            style: format!("position:relative; display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:100vh; background-color:{BG}; color:{TEXT}; font-family:{MONO}; padding:24px; overflow:hidden;"),

            {grid_overlay()}

            // Hex decoration floating right, as in 02_restore_wallet_options_desktop.svg
            div {
                style: "position:absolute; top:120px; right:80px; pointer-events:none;",
                {hex_pattern(6, 2, 0.3)}
            }

            div { style: "position:relative; z-index:1; display:flex; flex-direction:column; align-items:center; width:100%;",
                {glow_title(32, "WALLET SETUP")}
                p {
                    style: format!("color:{DIM}; margin:0 0 40px 0; font-size:10px; letter-spacing:3px;"),
                    "CHOOSE YOUR APPROACH"
                }

                button {
                    style: format!("width:100%; max-width:340px; padding:16px 20px; background-color:{PANEL}; color:{TEXT}; font-size:13px; font-weight:bold; border-radius:6px; border:1px solid {MINT}; cursor:pointer; font-family:{MONO}; text-align:left; margin-bottom:16px; box-shadow:0 0 12px rgba(100,255,218,0.12);"),
                    onclick: move |_| { nav.push(Route::CreateNewWallet {}); },
                    r#type: "button",
                    div { style: "display:flex; align-items:center; gap:14px;",
                        span { style: format!("color:{MINT}; font-size:18px;"), "+" }
                        span { style: "font-weight:bold; letter-spacing:1px;", "CREATE NEW WALLET" }
                    }
                    p { style: format!("margin:6px 0 0 32px; color:{DIM}; font-size:9px; letter-spacing:1px;"), "Generate a fresh wallet with new keys" }
                }

                button {
                    style: format!("width:100%; max-width:340px; padding:16px 20px; background-color:{PANEL}; color:{TEXT}; font-size:13px; font-weight:bold; border-radius:6px; border:1px solid {HAIR}; cursor:pointer; font-family:{MONO}; text-align:left; margin-bottom:16px;"),
                    onclick: move |_| { nav.push(Route::RestoreFromSeed {}); },
                    r#type: "button",
                    div { style: "display:flex; align-items:center; gap:14px;",
                        span { style: "color:#ffd700; font-size:18px;", "▣" }
                        span { style: "font-weight:bold; letter-spacing:1px;", "RESTORE FROM SEED PHRASE" }
                    }
                    p { style: format!("margin:6px 0 0 32px; color:{DIM}; font-size:9px; letter-spacing:1px;"), "Recover wallet using 12 or 24-word seed" }
                }

                button {
                    style: format!("width:100%; max-width:340px; padding:16px 20px; background-color:{PANEL}; color:{TEXT}; font-size:13px; font-weight:bold; border-radius:6px; border:1px solid {HAIR}; cursor:pointer; font-family:{MONO}; text-align:left;"),
                    onclick: move |_| { nav.push(Route::RestoreFromPrivateKey {}); },
                    r#type: "button",
                    div { style: "display:flex; align-items:center; gap:14px;",
                        span { style: "color:#ffd700; font-size:18px;", "⚿" }
                        span { style: "font-weight:bold; letter-spacing:1px;", "RESTORE FROM PRIVATE KEY" }
                    }
                    p { style: format!("margin:6px 0 0 32px; color:{DIM}; font-size:9px; letter-spacing:1px;"), "Import wallet using raw private key" }
                }
            }
        }
    }
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

    rsx! {
        div {
            style: format!("position:relative; display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:100vh; background-color:{BG}; color:{TEXT}; font-family:{MONO}; padding:24px; overflow:hidden;"),

            {grid_overlay()}

            // Back button
            button {
                style: format!("position:absolute; top:24px; left:24px; z-index:2; {}", back_button_style()),
                onclick: move |_| { nav.push(Route::RestoreWalletOptions {}); },
                r#type: "button",
                "← BACK"
            }

            div { style: "position:relative; z-index:1; display:flex; flex-direction:column; align-items:center; width:100%;",
                {glow_title(28, "CREATE NEW WALLET")}
                p { style: format!("color:{DIM}; font-size:10px; letter-spacing:2px; margin:0 0 32px 0;"), "GENERATE A NEW ON-CHAIN IDENTITY" }

                form {
                    style: card_style(),
                    onsubmit: create_wallet,

                    div { style: format!("display:flex; flex-direction:column; gap:8px;"),
                        div { style: field_label_style(), "WALLET NAME" }
                        input {
                            style: field_style(),
                            placeholder: "Enter a wallet name...",
                            value: "{wallet_name}",
                            oninput: move |event| wallet_name.set(event.value())
                        }
                    }

                    div { style: format!("display:flex; flex-direction:column; gap:8px;"),
                        div { style: field_label_style(), "PASSWORD" }
                        input {
                            style: field_style(),
                            r#type: "password",
                            placeholder: "Enter wallet password...",
                            value: "{wallet_password}",
                            oninput: move |event| wallet_password.set(event.value())
                        }
                    }

                    button { style: submit_style(), r#type: "submit", "CREATE WALLET" }

                    // Status box (fixed in design 03, live here)
                    div {
                        style: format!("padding:12px; border:1px solid {HAIR}; border-radius:6px; background-color:{BG}; color:{DIM}; font-size:10px; letter-spacing:1px; text-align:center;"),
                        if wallet_msg.read().is_empty() { "WALLET CREATION WILL BEGIN UPON SUBMISSION" } else { "{wallet_msg.read()}" }
                    }
                }
            }
        }
    }
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

    rsx! {
        div {
            style: format!("position:relative; display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:100vh; background-color:{BG}; color:{TEXT}; font-family:{MONO}; padding:24px; overflow:hidden;"),

            {grid_overlay()}

            button {
                style: format!("position:absolute; top:24px; left:24px; z-index:2; {}", back_button_style()),
                onclick: move |_| { nav.push(Route::RestoreWalletOptions {}); },
                r#type: "button",
                "← BACK"
            }

            div { style: "position:relative; z-index:1; display:flex; flex-direction:column; align-items:center; width:100%;",
                {glow_title(28, "RESTORE FROM SEED")}
                p { style: format!("color:{DIM}; font-size:10px; letter-spacing:2px; margin:0 0 32px 0;"), "ENTER YOUR 12 OR 24-WORD SEED PHRASE" }

                form {
                    style: card_style(),
                    onsubmit: restore_wallet,

                    div { style: format!("display:flex; flex-direction:column; gap:8px;"),
                        div { style: field_label_style(), "WALLET NAME" }
                        input {
                            style: field_style(),
                            placeholder: "Enter a wallet name...",
                            value: "{wallet_name}",
                            oninput: move |event| wallet_name.set(event.value())
                        }
                    }

                    div { style: format!("display:flex; flex-direction:column; gap:8px;"),
                        div { style: field_label_style(), "PASSWORD" }
                        input {
                            style: field_style(),
                            r#type: "password",
                            placeholder: "Enter wallet password...",
                            value: "{wallet_password}",
                            oninput: move |event| wallet_password.set(event.value())
                        }
                    }

                    div { style: format!("display:flex; flex-direction:column; gap:8px;"),
                        div { style: field_label_style(), "SEED PHRASE" }
                        textarea {
                            style: format!("{} height:90px; resize:vertical;", field_style()),
                            placeholder: "word1 word2 word3 ...",
                            value: "{wallet_seed}",
                            oninput: move |event| wallet_seed.set(event.value())
                        }
                    }

                    button { style: submit_style(), r#type: "submit", "RESTORE WALLET" }
                }
            }
        }
    }
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

    rsx! {
        div {
            style: format!("position:relative; display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:100vh; background-color:{BG}; color:{TEXT}; font-family:{MONO}; padding:24px; overflow:hidden;"),

            {grid_overlay()}

            button {
                style: format!("position:absolute; top:24px; left:24px; z-index:2; {}", back_button_style()),
                onclick: move |_| { nav.push(Route::RestoreWalletOptions {}); },
                r#type: "button",
                "← BACK"
            }

            div { style: "position:relative; z-index:1; display:flex; flex-direction:column; align-items:center; width:100%;",
                {glow_title(28, "RESTORE FROM PRIVATE KEY")}
                p { style: format!("color:{DIM}; font-size:10px; letter-spacing:2px; margin:0 0 24px 0;"), "IMPORT AN EXISTING RAW KEY" }

                // Amber warning banner
                div {
                    style: format!("width:100%; max-width:360px; padding:12px; border:1px solid rgba(255,215,0,0.27); border-radius:6px; background-color:rgba(255,215,0,0.05); margin-bottom:28px; text-align:center; position:relative; z-index:1;"),
                    p { style: "color:#ffd700; font-size:10px; letter-spacing:2px; margin:0;", "⚠ THIS KEY GRANTS FULL CONTROL OF FUNDS · NEVER SHARE IT" }
                }

                form {
                    style: card_style(),
                    onsubmit: restore_wallet,

                    div { style: format!("display:flex; flex-direction:column; gap:8px;"),
                        div { style: field_label_style(), "WALLET NAME" }
                        input {
                            style: field_style(),
                            placeholder: "Enter a wallet name...",
                            value: "{wallet_name}",
                            oninput: move |event| wallet_name.set(event.value())
                        }
                    }

                    div { style: format!("display:flex; flex-direction:column; gap:8px;"),
                        div { style: field_label_style(), "PASSWORD" }
                        input {
                            style: field_style(),
                            r#type: "password",
                            placeholder: "Enter wallet password...",
                            value: "{wallet_password}",
                            oninput: move |event| wallet_password.set(event.value())
                        }
                    }

                    div { style: format!("display:flex; flex-direction:column; gap:8px;"),
                        div { style: field_label_style(), "PRIVATE KEY" }
                        textarea {
                            style: format!("{} height:60px; resize:vertical;", field_style()),
                            placeholder: "Enter your private key...",
                            value: "{wallet_private_key}",
                            oninput: move |event| wallet_private_key.set(event.value())
                        }
                    }

                    button { style: submit_style(), r#type: "submit", "RESTORE WALLET" }
                }
            }
        }
    }
}
