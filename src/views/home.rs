use crate::{
    Route, WALLET,
    database::db_fns::db_add_contact,
    views::{
        chat_view::chat_shell,
        sidebar::{SimpleSidebar, SimpleSidebarProps},
        theme::{copy_to_clipboard, grid_overlay, AMBER, BG, DIM, HAIR, MINT, MONO, PANEL, SUB, TEXT},
        DbContact,
    },
    wallet::wallet_fns::wallet_get_seed,
};
use crate::DB;
use dioxus::{logger::tracing::info, prelude::*};

/// Home is a thin wrapper over the unified chat shell (empty state when no
/// conversation is selected).
#[component]
pub fn Home() -> Element {
    chat_shell(None)
}

fn field_style() -> String {
    format!(
        "width:100%; box-sizing:border-box; padding:14px 18px; background-color:{BG}; border:1px solid {HAIR}; border-radius:6px; color:{TEXT}; outline:none; font-size:16px; font-family:{MONO};"
    )
}

fn field_label_style() -> String {
    format!("font-size:11px; color:{DIM}; letter-spacing:2px; margin-bottom:8px;")
}

fn submit_style() -> String {
    format!(
        "padding:16px 24px; background:none; color:{MINT}; font-size:14px; font-weight:bold; border-radius:6px; border:1px solid {MINT}; cursor:pointer; font-family:{MONO}; letter-spacing:3px; box-shadow:0 0 12px rgba(100,255,218,0.12); width:100%;"
    )
}

fn page_main(
    title: &'static str,
    on_back: impl Fn() + 'static,
    back: Element,
    body: Element,
) -> Element {
    rsx! {
        div {
            style: format!("display:flex; min-height:100vh; background-color:{BG}; font-family:{MONO}; position:relative;"),
            {grid_overlay()}
            {back}
            div {
                style: format!("flex:1; min-width:0; display:flex; flex-direction:column; padding:24px 28px; position:relative; z-index:1;"),
                div {
                    style: format!("display:flex; align-items:center; gap:16px; margin-bottom:24px;"),
                    button {
                        style: format!("font-size:14px; color:{DIM}; background:none; border:1px solid {HAIR}; border-radius:6px; padding:10px 16px; cursor:pointer; font-family:{MONO}; letter-spacing:2px;"),
                        onclick: move |_| on_back(),
                        "←"
                    }
                    h1 {
                        style: format!("font-size:18px; font-weight:bold; margin:0; color:{TEXT}; letter-spacing:3px; font-family:{MONO};"),
                        "{title}"
                    }
                }
                div { style: "flex:1; display:flex; align-items:center; justify-content:center;",
                    {body}
                }
            }
        }
    }
}

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
#[component]
pub fn AddContact() -> Element {
    let nav = navigator();

    let mut contact_address = use_signal(|| String::new());
    let mut contact_name = use_signal(|| String::new());
    let mut contact_ret_msg = use_signal(|| String::new());

    let add_contact = move |_: FormEvent| async move {
        let new_contact = DbContact {
            name: contact_name.read().clone(),
            address: contact_address.read().clone(),
        };

        let wallet_address = match &*WALLET.read() {
            Some(wallet) => Some(wallet.read().await.get_address().await),
            None => {
                info!("Error reading wallet");
                None
            }
        };

        if let Some(wallet_address) = wallet_address {
            if !new_contact.name.is_empty()
                && !new_contact.address.is_empty()
                && new_contact.address != wallet_address
            {
                if let Some(db) = &*DB.read() {
                    db_add_contact(db, &mut contact_ret_msg, new_contact).await;
                }
            } else {
                contact_ret_msg.set("Name / address cannot be empty".to_string());
            }
        } else {
            contact_ret_msg.set("Error reading wallet address".to_string());
        }
    };

    let body = rsx! {
        div {
            style: format!("display:flex; flex-direction:column; gap:18px; width:100%; max-width:440px; background-color:{PANEL}; padding:30px; border-radius:8px; border:1px solid {HAIR}; position:relative; z-index:1;"),
            h2 {
                style: format!("font-size:18px; font-weight:bold; margin:0; color:{MINT}; letter-spacing:4px; text-align:center; text-shadow:0 0 14px rgba(100,255,218,0.35);"),
                "ADD CONTACT"
            }
            div {
                style: format!("font-size:11px; color:{DIM}; letter-spacing:2px; text-align:center; margin-bottom:6px;"),
                "STORED IN YOUR ADDRESS BOOK · NEVER BROADCAST ON-CHAIN"
            }

            form {
                onsubmit: add_contact,
                div { style: "margin-bottom:20px;",
                    div { style: field_label_style(), "CONTACT NAME" }
                    input {
                        style: field_style(),
                        placeholder: "Enter contact name...",
                        value: "{contact_name}",
                        oninput: move |event| contact_name.set(event.value()),
                    }
                }
                div { style: "margin-bottom:20px;",
                    div { style: field_label_style(), "WALLET ADDRESS" }
                    input {
                        style: field_style(),
                        placeholder: "Enter a wallet address...",
                        value: "{contact_address}",
                        oninput: move |event| contact_address.set(event.value()),
                    }
                }
                div { style: "display:flex; justify-content:center;",
                    button { style: submit_style(), r#type: "submit", "ADD CONTACT" }
                }
                div {
                    style: format!("margin-top:16px; padding:10px; border:1px solid {HAIR}; border-radius:6px; color:{AMBER}; font-size:13px; text-align:center; min-height:18px;"),
                    "{contact_ret_msg.read()}"
                }
            }
        }
    };

    let back = rsx! {
        {SimpleSidebar(SimpleSidebarProps { subtitle: "v0.0.1 · ADD CONTACT".to_string() })}
    };

    page_main("ADD CONTACT", move || { nav.push(Route::Home {}); }, back, body)
}

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
#[component]
pub fn ViewSeed() -> Element {
    let nav = navigator();

    let mut user_password = use_signal(|| String::new());
    let mut seed_phrase = use_signal(|| String::new());
    let mut copy_feedback = use_signal(|| String::new());

    let get_seed_phrase = move |_: FormEvent| async move {
        let entered_password = user_password.read().clone();

        match &*DB.read() {
            Some(db) => wallet_get_seed(db, entered_password, &mut seed_phrase).await,
            None => {
                info!("DB not accessible");
                seed_phrase.set("DB not accessible".to_string());
            }
        };
    };

    let seed = seed_phrase();
    let words: Vec<String> = seed.split_whitespace().map(|w| w.to_string()).collect();

    let body = rsx! {
        div {
            style: format!("display:flex; flex-direction:column; gap:18px; width:100%; max-width:480px; background-color:{PANEL}; padding:30px; border-radius:8px; border:1px solid rgba(255,215,0,0.27); position:relative; z-index:1;"),
            // Warning banner
            div {
                style: format!("padding:12px 14px; border:1px solid rgba(255,215,0,0.27); border-radius:6px; background-color:rgba(255,215,0,0.05); text-align:center;"),
                span { style: format!("color:{AMBER}; font-size:12px; letter-spacing:2px;"), "⚠️ KEEP YOUR SEED PHRASE SECRET AND SECURE" }
            }

            // Seed display
            if seed.is_empty() {
                div {
                    style: format!("padding:22px; text-align:center; font-size:13px; color:{DIM}; letter-spacing:2px; border:1px dashed {HAIR}; border-radius:6px;"),
                    "ENCRYPTED · ENTER PASSWORD TO DECRYPT"
                }
            } else if words.len() < 12 {
                // error / unexpected payload — show raw
                div {
                    style: format!("padding:14px; border:1px solid rgba(255,215,0,0.27); border-radius:6px; color:{AMBER}; font-size:14px; text-align:center;"),
                    "{seed}"
                }
            } else {
                div {
                    style: "display:grid; grid-template-columns:repeat(3,1fr); gap:10px;",
                    for (i, word) in words.iter().enumerate() {
                        div {
                            style: format!("padding:12px; border:1px solid {HAIR}; border-radius:6px; background-color:{BG}; font-size:14px;"),
                            span { style: format!("color:{DIM}; margin-right:8px;"), "{i+1}." }
                            span { style: format!("color:{TEXT};"), "{word}" }
                        }
                    }
                }
                div { style: "display:flex; align-items:center; gap:12px;",
                    div {
                        style: format!("flex:1; padding:12px 14px; border-radius:6px; background-color:{BG}; border:1px solid {MINT}; cursor:pointer; text-align:center;"),
                        onclick: move |_| {
                            match copy_to_clipboard(&seed) {
                                Ok(_) => copy_feedback.set("✓ Copied!".to_string()),
                                Err(e) => copy_feedback.set(format!("Error: {:?}", e)),
                            }
                        },
                        span { style: format!("color:{MINT}; font-size:13px; letter-spacing:2px;"), "📋 COPY SEED" }
                    }
                    span { style: format!("font-size:12px; color:{DIM}; min-width:70px;"), "{copy_feedback.read()}" }
                }
            }

            // Password form
            form {
                onsubmit: get_seed_phrase,
                div { style: "margin-bottom:16px;",
                    div { style: field_label_style(), "ACCOUNT PASSWORD" }
                    input {
                        style: field_style(),
                        placeholder: "Enter account password...",
                        value: "{user_password}",
                        oninput: move |event| user_password.set(event.value()),
                    }
                }
                button { style: submit_style(), r#type: "submit", "GET SEED" }
            }
            div {
                style: format!("font-size:11px; color:{SUB}; letter-spacing:2px; text-align:center;"),
                "DECRYPTS LOCALLY · NEVER LEAVES THIS DEVICE"
            }
        }
    };

    let back = rsx! {
        {SimpleSidebar(SimpleSidebarProps { subtitle: "v0.0.1 · VIEW SEED".to_string() })}
    };

    page_main("GET SEED PHRASE", move || { nav.push(Route::Home {}); }, back, body)
}
