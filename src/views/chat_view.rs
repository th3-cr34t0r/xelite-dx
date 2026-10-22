use crate::{
    DB, IS_READY, WALLET,
    database::db_fns::{db_read_messages, db_store_init_message, db_update_status_topoheight},
    views::{
        sidebar::{MessengerSidebar, MessengerSidebarProps, build_contact_rows},
        theme::{
            avatar, fmt_clock, fmt_date, grid_overlay, hex_pattern, short_addr, AMBER, BG, DIM,
            HAIR, MINT, MONO, PANEL, ROWHAIR, SUB, TEXT,
        },
        ContactRow, DbMessage,
    },
    wallet::{utils::NODE_ENDPOINT, wallet_fns::{DEV_FEE_AMOUNT, wallet_send_message}},
};
use crate::Route;
use chrono::Utc;
use dioxus::{logger::tracing::info, prelude::*};

/// Unified messenger shell: sidebar on the left (conversations + settings),
/// chat panel on the right. `active` = Some((name, address)) opens that chat;
/// None renders the empty state. This is the only desktop main page, so there
/// is always a way back: the conversation list + settings stay visible.
#[allow(clippy::redundant_closure, clippy::await_holding_invalid_type)]
pub fn chat_shell(active: Option<(String, String)>) -> Element {
    let mut topoheight = use_signal(|| 0i64);
    let mut messages_from_db = use_signal(|| vec![DbMessage::default()]);
    let mut rows = use_signal(|| Vec::<ContactRow>::new());

    // reactive current chat address; resources restart when it changes
    let mut active_addr = use_signal(String::new);
    let want_addr = active
        .as_ref()
        .map(|(_, a)| a.clone())
        .unwrap_or_default();
    if *active_addr.read() != want_addr {
        active_addr.set(want_addr);
    }

    // load the messages from the database
    let mut msg_handle = use_future(move || async move {
        let addr = active_addr.read().clone();
        if addr.is_empty() {
            return;
        }
        match &*DB.read() {
            Some(db) => db_read_messages(db, addr, &mut messages_from_db).await,
            None => info!("Error reading DB"),
        };
    });

    // sidebar conversation list
    let mut rows_handle = use_resource(move || async move {
        if let Some(db) = &*DB.read() {
            *rows.write() = build_contact_rows(db).await;
        }
    });

    // message signal
    let mut send_msg = use_signal(String::new);
    let info = use_signal(String::new);
    let mut wallet_is_ready = use_signal(|| true);

    // sidebar state
    let search = use_signal(String::new);
    let mut full_address = use_signal(String::new);
    let mut balance = use_signal(String::new);
    let mut online = use_signal(|| false);

    let submit_tx_message = move |_: FormEvent| {
        let addr = active_addr.read().clone();
        let message = send_msg.read().clone();
        if addr.is_empty() || message.is_empty() {
            return;
        }
        send_msg.set("".to_string());
        wallet_is_ready.set(false);

        let topo = topoheight();
        let mut handle = msg_handle;
        let mut info_sig = info;
        let mut rows_h = rows_handle;
        dioxus::prelude::spawn(async move {
            *IS_READY.write().write().await = false;
            wallet_send_message(addr, topo, message, &mut handle, &mut info_sig).await;
            rows_h.restart();
        });
    };

    // wallet init + pool for new app events
    use_future(move || async move {
        if let Some(wallet) = &*WALLET.read() {
            // one-time init: address, online mode, balance
            *full_address.write() = wallet.read().await.get_address().await;
            match wallet
                .write()
                .await
                .set_online(NODE_ENDPOINT.to_string())
                .await
            {
                Ok(_) => online.set(true),
                Err(e) => {
                    info!("Error opening wallet connection: {}", e);
                    online.set("Wallet is already in online mode" == e.to_string());
                }
            }
            if let Ok(ret_balance) = wallet.write().await.get_balance().await {
                balance.set(ret_balance);
            }

            loop {
                if let Some(wallet) = &*WALLET.read() {
                    // retrieve wallet state
                    topoheight.set(wallet.read().await.topoheight as i64);
                    balance.set(wallet.read().await.balance.clone());
                    online.set(wallet.read().await.is_online);

                    // check for app events
                    wallet.write().await.backgroud_daemon().await;

                    // store received messages in db
                    while let Some(tx) = wallet.write().await.rx_messages.pop() {
                        db_store_init_message(tx).await;
                        msg_handle.restart();
                        rows_handle.restart();
                    }

                    while let Some(message) = wallet.write().await.confirmed_messages.pop() {
                        db_update_status_topoheight(message).await;

                        *IS_READY.write().write().await = true;
                        wallet_is_ready.set(true);
                        msg_handle.restart();
                        rows_handle.restart();
                    }
                }
            }
        }
    });

    let nav = navigator();
    let subtitle = format!(
        "{} · v0.0.1",
        short_addr(&full_address.read().clone())
    );
    let addr_for_sidebar = active.as_ref().map(|(_, a)| a.clone());

    let chat_col = if let Some((name, address)) = active.as_ref() {
        let divider = format!("{} · TOPO #{}", fmt_date(Utc::now().timestamp()), topoheight());
        rsx! {
            // Chat column (single wrapper div — otherwise children land in the outer flex row)
            div {
                style: format!("flex:1; min-width:0; display:flex; flex-direction:column; position:relative; z-index:1;"),

                // Chat header
                div {
                style: format!("display:flex; align-items:center; gap:14px; padding:16px 24px; background-color:{PANEL}; border-bottom:1px solid {HAIR};"),
                {avatar(&name, 44, true)}
                div { style: "display:flex; flex-direction:column; gap:5px; min-width:0;",
                    span { style: format!("font-size:18px; color:{TEXT}; letter-spacing:2px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;"), "{name}" }
                    span { style: format!("font-size:12px; color:{DIM}; letter-spacing:1px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;"), "● ENCRYPTED · KEY {short_addr(address)}" }
                }
                div { style: "margin-left:auto; display:flex; gap:16px; align-items:baseline;",
                    if wallet_is_ready() {
                        span { style: format!("font-size:12px; color:{MINT}; letter-spacing:1px;"), "● ONLINE" }
                    } else {
                        span { style: format!("font-size:12px; color:{AMBER}; letter-spacing:1px;"), "● PENDING TX" }
                    }
                    span { style: format!("font-size:13px; color:{DIM}; letter-spacing:1px;"), "TOP #{topoheight()}" }
                }
            }

            // Date divider
            div {
                style: format!("text-align:center; padding:16px 0 6px; font-size:11px; color:{DIM}; letter-spacing:2px;"),
                "{divider}"
            }

            // Messages area
            div {
                style: "flex:1; overflow-y:auto; padding:16px 26px; display:flex; flex-direction:column-reverse;",
                {message_stream(&messages_from_db, &name)}
            }

            // Decrypting indicator
            if !wallet_is_ready() {
                div {
                    style: format!("padding:8px 26px; font-size:12px; color:{AMBER}; letter-spacing:1px;"),
                    "● {name} is decrypting…"
                }
            }

            // Input area
            form {
                style: format!("display:flex; align-items:center; gap:12px; padding:16px 24px; background-color:{PANEL}; border-top:1px solid {HAIR};"),
                onsubmit: submit_tx_message,
                span { style: format!("color:{MINT}; font-size:16px;"), ">" }
                input {
                    style: format!("flex:1; padding:14px 18px; background-color:{BG}; border:1px solid {HAIR}; border-radius:6px; color:{TEXT}; outline:none; font-size:16px; font-family:{MONO};"),
                    placeholder: "type a secure message...",
                    value: "{send_msg}",
                    oninput: move |event| send_msg.set(event.value()),
                    id: "message-input",
                    autofocus: true,
                    disabled: !wallet_is_ready(),
                }
                button {
                    style: format!("padding:14px 26px; background:none; color:{MINT}; border:1px solid {MINT}; border-radius:6px; cursor:pointer; font-family:{MONO}; font-size:13px; font-weight:bold; letter-spacing:3px; box-shadow:0 0 12px rgba(100,255,218,0.12); white-space:nowrap;"),
                    r#type: "submit",
                    disabled: !wallet_is_ready(),
                    "➤ SEND"
                }
            }

            // Footer note
            div {
                style: format!("padding:12px 24px; background-color:{BG}; text-align:center;"),
                if info().is_empty() {
                    span { style: format!("font-size:11px; color:{DIM}; letter-spacing:2px;"), "SENT AS ON-CHAIN TX · ENCRYPTED WITH RECIPIENT KEY · CONFIRMED AT TOPOHEIGHT" }
                } else {
                    span { style: format!("font-size:12px; color:{AMBER}; letter-spacing:1px;"), "{info()}" }
                }
            }
            }
        }
    } else {
        // Empty state: no conversation selected
        rsx! {
            div {
                style: format!("flex:1; min-width:0; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:16px; padding:40px; position:relative; z-index:1;"),
                {hex_pattern(2, 1, 0.5)}
                h1 {
                    style: format!("font-size:20px; font-weight:bold; margin:0; color:{MINT}; letter-spacing:5px; text-shadow:0 0 16px rgba(100,255,218,0.35);"),
                    "XELITE MESH"
                }
                span { style: format!("font-size:14px; color:{DIM}; letter-spacing:2px;"), "SELECT A CONVERSATION ON THE LEFT" }
                span { style: format!("font-size:12px; color:{SUB}; letter-spacing:1px;"), "…or add a new contact to start an encrypted chat" }
                button {
                    style: format!("padding:14px 26px; background:none; color:{MINT}; border:1px solid {MINT}; border-radius:6px; cursor:pointer; font-family:{MONO}; font-size:13px; font-weight:bold; letter-spacing:3px; box-shadow:0 0 12px rgba(100,255,218,0.12); margin-top:6px;"),
                    onclick: move |_| { nav.push(Route::AddContact {}); },
                    "＋ ADD CONTACT"
                }
            }
        }
    };

    rsx!(
        div {
            style: format!("display:flex; height:100vh; background-color:{BG}; font-family:{MONO}; position:relative;"),
            {grid_overlay()}
            {MessengerSidebar(
                MessengerSidebarProps {
                    subtitle,
                    full_address: full_address.read().clone(),
                    balance: balance.read().clone(),
                    online: online(),
                    topoheight: topoheight(),
                    search,
                    rows: rows.read().clone(),
                    active: addr_for_sidebar,
                },
            )}
            {chat_col}
        }
    )
}

#[component]
pub fn ChatView(name: String, address: String) -> Element {
    chat_shell(Some((name, address)))
}

fn message_stream(messages: &Signal<Vec<DbMessage>>, contact_name: &str) -> Element {
    let bubbles: Vec<Element> = messages
        .read()
        .iter()
        .rev()
        .filter_map(|msg| {
            let text = msg.message.clone()?;
            let clock = fmt_clock(msg.timestamp);
            let hash = short_addr(&msg.hash);
            let fee = format!("{} XEL", msg.fee + DEV_FEE_AMOUNT);
            let status = msg.status.clone();
            let topo = msg.topoheight;

            if msg.direction == "Outgoing" {
                let tx = format!("tx {hash}");
                let top = format!("TOP #{topo}");
                Some(rsx! {
                    div {
                        style: format!("margin-bottom:16px; padding:16px 20px; border-radius:8px; background-color:rgba(100,255,218,0.04); border:1px solid rgba(100,255,218,0.27); max-width:64%; align-self:flex-end;"),
                        div { style: "display:flex; justify-content:space-between; gap:16px; margin-bottom:8px;",
                            span { style: format!("font-size:11px; color:{MINT}; letter-spacing:2px;"), "> YOU" }
                            if status == "Pending" {
                                span { style: format!("font-size:11px; color:{AMBER}; letter-spacing:1px;"), "● PENDING" }
                            } else {
                                span { style: format!("font-size:11px; color:{MINT}; letter-spacing:1px;"), "✓ CONFIRMED" }
                            }
                        }
                        p { style: format!("margin:0 0 10px 0; font-size:17px; color:{TEXT}; line-height:1.5; text-align:right;"), "{text}" }
                        div { style: "display:flex; justify-content:flex-end; gap:10px; align-items:baseline; flex-wrap:wrap;",
                            span { style: format!("font-size:11px; color:{DIM};"), "{clock}" }
                            if !hash.is_empty() { span { style: format!("font-size:11px; color:{DIM};"), "{tx}" } }
                            if topo > 0 { span { style: format!("font-size:11px; color:{DIM};"), "{top}" } }
                            if status == "Pending" { span { style: format!("font-size:11px; color:{AMBER};"), "{fee}" } }
                        }
                    }
                })
            } else {
                let tx = format!("tx {hash}");
                let top = format!("✓ TOP #{topo}");
                let st = status.clone();
                let who = format!("< {contact_name}");
                Some(rsx! {
                    div {
                        style: format!("margin-bottom:16px; padding:16px 20px; border-radius:8px; background-color:{PANEL}; border:1px solid {ROWHAIR}; max-width:64%; align-self:flex-start;"),
                        div { style: "display:flex; justify-content:space-between; gap:16px; margin-bottom:8px;",
                            span { style: format!("font-size:11px; color:{MINT}; letter-spacing:2px;"), "{who}" }
                            span { style: format!("font-size:11px; color:{DIM}; letter-spacing:1px;"), "{st}" }
                        }
                        p { style: format!("margin:0 0 10px 0; font-size:17px; color:{SUB}; line-height:1.5;"), "{text}" }
                        div { style: "display:flex; gap:10px; align-items:baseline; flex-wrap:wrap;",
                            span { style: format!("font-size:11px; color:{DIM};"), "{clock}" }
                            if !hash.is_empty() { span { style: format!("font-size:11px; color:{DIM};"), "{tx}" } }
                            if topo > 0 { span { style: format!("font-size:11px; color:{DIM};"), "{top}" } }
                        }
                    }
                })
            }
        })
        .collect();

    rsx! {
        if bubbles.is_empty() {
            div {
                style: format!("text-align:center; padding:40px 0; font-size:13px; color:{DIM}; letter-spacing:2px;"),
                "NO MESSAGES YET — SAY HELLO"
            }
        } else {
            for b in bubbles {
                {b}
            }
        }
    }
}
