use dioxus::prelude::*;
use sqlx::SqlitePool;

use crate::database::db_fns::{db_read_contacts_list, db_read_last_message};
use crate::views::theme::{
    avatar, copy_to_clipboard, fmt_clock, fmt_k, label, AMBER, BG, DIM, HAIR, MINT, MONO, PANEL,
    ROW, SUB, TEXT,
};
use crate::views::ContactRow;
use crate::Route;

/// Contacts + last-message preview, assembled for conversation rows
pub async fn build_contact_rows(db: &SqlitePool) -> Vec<ContactRow> {
    let contacts = db_read_contacts_list(db).await;
    let mut rows = Vec::with_capacity(contacts.len());
    for c in contacts {
        match db_read_last_message(db, c.address.clone()).await {
            Some((message, timestamp, status, topo)) => rows.push(ContactRow {
                name: c.name,
                address: c.address,
                preview: message,
                timestamp,
                status,
                topo,
            }),
            None => rows.push(ContactRow {
                name: c.name,
                address: c.address,
                ..Default::default()
            }),
        }
    }
    rows
}

fn sidebar_header(subtitle: &str) -> Element {
    let subtitle = subtitle.to_string();
    rsx! {
        div { style: "padding:24px 20px 16px;",
            h1 {
                style: format!("font-size:26px; font-weight:bold; margin:0; color:{MINT}; letter-spacing:5px; font-family:{MONO}; text-shadow:0 0 16px rgba(100,255,218,0.35);"),
                "XELITE"
            }
            div {
                style: format!("font-size:12px; color:{DIM}; letter-spacing:1px; margin-top:6px; font-family:{MONO}; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;"),
                "{subtitle}"
            }
        }
    }
}

fn settings_row_style() -> String {
    format!(
        "display:flex; align-items:center; gap:12px; padding:12px 20px; cursor:pointer; border-left:2px solid transparent; font-family:{MONO};"
    )
}

fn settings_icon() -> String {
    format!("font-size:15px; color:{MINT}; width:18px; text-align:center;")
}

fn settings_label() -> String {
    format!("color:{SUB}; font-size:13px; letter-spacing:2px;")
}

fn sidebar_footer() -> Element {
    rsx! {
        div {
            style: format!("border-top:1px solid {HAIR}; padding:14px 20px; font-size:12px; color:{DIM}; letter-spacing:1px; font-family:{MONO};"),
            "Xelite v0.0.1"
        }
    }
}

/// Slim sidebar for secondary pages (designs 07/08)
#[derive(Clone, PartialEq, Props)]
pub struct SimpleSidebarProps {
    pub subtitle: String,
}

#[component]
pub fn SimpleSidebar(props: SimpleSidebarProps) -> Element {
    let nav = navigator();
    rsx! {
        div {
            style: format!("width:240px; min-width:240px; display:flex; flex-direction:column; background-color:{PANEL}; border-right:1px solid {HAIR}; font-family:{MONO}; position:relative; z-index:2;"),
            {sidebar_header(&props.subtitle)}
            div { style: settings_row_style(), onclick: move |_| { nav.push(Route::AddContact {}); },
                span { style: settings_icon(), "＋" }
                span { style: settings_label(), "ADD CONTACT" }
            }
            div { style: settings_row_style(), onclick: move |_| { nav.push(Route::ViewSeed {}); },
                span { style: settings_icon(), "🔑" }
                span { style: settings_label(), "VIEW SEED" }
            }
            div { style: "margin-top:auto;", }
            {sidebar_footer()}
        }
    }
}

/// Unified messenger sidebar: Xelis balance on top, conversation list,
/// settings section (add contact / view seed / copy address), network status.
#[derive(Clone, PartialEq, Props)]
pub struct MessengerSidebarProps {
    pub subtitle: String,
    pub full_address: String,
    pub balance: String,
    pub online: bool,
    pub topoheight: i64,
    pub search: Signal<String>,
    pub rows: Vec<ContactRow>,
    pub active: Option<String>,
}

#[component]
pub fn MessengerSidebar(mut props: MessengerSidebarProps) -> Element {
    let nav = navigator();
    let mut copy_feedback = use_signal(String::new);

    rsx! {
        div {
            style: format!("width:300px; min-width:300px; display:flex; flex-direction:column; background-color:{PANEL}; border-right:1px solid {HAIR}; font-family:{MONO}; position:relative; z-index:2; overflow-y:auto;"),
            {sidebar_header(&props.subtitle)}

            // Xelis balance — top of the sidebar
            div {
                style: format!("margin:0 20px 16px; padding:14px 16px; background-color:{BG}; border:1px solid {HAIR}; border-radius:6px;"),
                {label("XELIS BALANCE", "margin-bottom:8px;")}
                div { style: "display:flex; justify-content:space-between; align-items:baseline;",
                    if props.balance.is_empty() {
                        span { style: format!("color:{AMBER}; font-size:14px; letter-spacing:1px;"), "SYNCING…" }
                    } else {
                        span { style: format!("color:{MINT}; font-size:20px; text-shadow:0 0 12px rgba(100,255,218,0.3);"), "{props.balance} XEL" }
                    }
                    if props.online {
                        span { style: format!("color:{MINT}; font-size:13px; letter-spacing:1px;"), "● ON" }
                    } else {
                        span { style: format!("color:{AMBER}; font-size:13px; letter-spacing:1px;"), "● OFF" }
                    }
                }
            }

            // Search
            div {
                style: format!("margin:0 20px 14px; display:flex; align-items:center; gap:8px; background-color:{BG}; border:1px solid {HAIR}; border-radius:6px; padding:10px 14px;"),
                span { style: format!("color:{DIM}; font-size:15px;"), "⌕" }
                input {
                    style: format!("flex:1; background:none; border:none; outline:none; color:{TEXT}; font-size:13px; letter-spacing:2px; font-family:{MONO};"),
                    placeholder: "SEARCH CONTACTS",
                    value: "{props.search}",
                    oninput: move |event| props.search.set(event.value()),
                }
            }

            // Conversation list
            div {
                style: format!("display:flex; justify-content:space-between; align-items:center; padding:4px 20px 8px;"),
                span { style: format!("font-size:11px; color:{DIM}; letter-spacing:2px;"), "CONVERSATIONS" }
                span { style: format!("font-size:12px; color:{MINT}; letter-spacing:1px;"), "{props.rows.len()}" }
            }
            div { style: "padding:0 14px; overflow-y:auto; min-height:0;",
                {ContactRows(
                    ContactRowsProps {
                        rows: props.rows.clone(),
                        query: props.search.read().clone(),
                        active: props.active.clone(),
                    },
                )}
            }

            // Settings section
            div { style: "margin:14px 20px 4px;", {label("SETTINGS", "")} }
            div { style: settings_row_style(), onclick: move |_| { nav.push(Route::AddContact {}); },
                span { style: settings_icon(), "＋" }
                span { style: settings_label(), "ADD CONTACT" }
            }
            div { style: settings_row_style(), onclick: move |_| { nav.push(Route::ViewSeed {}); },
                span { style: settings_icon(), "🔑" }
                span { style: settings_label(), "VIEW SEED" }
            }
            div { style: settings_row_style(),
                onclick: move |_| {
                    let addr = props.full_address.clone();
                    if addr.is_empty() {
                        copy_feedback.set("Address not loaded yet".to_string());
                        return;
                    }
                    match copy_to_clipboard(&addr) {
                        Ok(_) => copy_feedback.set("✓ Copied!".to_string()),
                        Err(e) => copy_feedback.set(format!("Error: {:?}", e)),
                    }
                },
                span { style: settings_icon(), "📋" }
                span { style: settings_label(), "COPY ADDRESS" }
            }
            div {
                style: format!("margin:2px 20px 14px 50px; min-height:18px; font-size:12px; color:{DIM}; letter-spacing:1px;"),
                "{copy_feedback.read()}"
            }

            // Network status
            div {
                style: format!("margin-top:auto; border-top:1px solid {HAIR}; padding:16px 20px 0;"),
                {label("NETWORK STATUS", "margin-bottom:12px;")}
                div { style: "display:flex; gap:28px; align-items:baseline;",
                    div {
                        div { style: format!("font-size:21px; color:{MINT}; font-family:{MONO};"), "{fmt_k(props.topoheight)}" }
                        {label("TOPOHEIGHT", "")}
                    }
                    div {
                        div {
                            style: format!("font-size:15px; font-family:{MONO}; {}", if props.online { format!("color:{MINT}") } else { format!("color:{AMBER}") }),
                            if props.online { "● ONLINE" } else { "● OFFLINE" }
                        }
                        {label("NODE", "")}
                    }
                }
            }
            {sidebar_footer()}
        }
    }
}

/// Conversation row list, search-filtered
#[derive(Clone, PartialEq, Props)]
pub struct ContactRowsProps {
    pub rows: Vec<ContactRow>,
    pub query: String,
    pub active: Option<String>,
}

#[component]
pub fn ContactRows(props: ContactRowsProps) -> Element {
    let nav = navigator();
    let q = props.query.to_lowercase();

    let rows: Vec<Element> = props
        .rows
        .iter()
        .filter(|r| r.name.to_lowercase().contains(&q))
        .map(|row| {
            let active = props.active.as_deref() == Some(row.address.as_str());
            let border = if active { format!("1px solid {MINT}") } else { format!("1px solid {HAIR}") };
            let preview = if row.preview.is_empty() { "—".to_string() } else { row.preview.clone() };
            let clock = fmt_clock(row.timestamp);
            let av = avatar(&row.name, 40, active);
            let name = row.name.clone();
            let address = row.address.clone();

            let status_el = if row.status == "Pending" {
                rsx! { span { style: format!("font-size:12px; color:{AMBER}; letter-spacing:1px;"), "● PENDING" } }
            } else if row.topo > 0 {
                let top = format!("✓ TOP #{}", row.topo);
                rsx! { span { style: format!("font-size:12px; color:{MINT}; letter-spacing:1px;"), "{top}" } }
            } else {
                let st = format!("✓ {}", row.status);
                rsx! { span { style: format!("font-size:12px; color:{DIM}; letter-spacing:1px;"), "{st}" } }
            };

            rsx! {
                div {
                    style: format!("display:flex; align-items:center; gap:12px; padding:14px; margin-bottom:10px; background-color:{ROW}; border:{border}; border-radius:6px; cursor:pointer; font-family:{MONO};"),
                    onclick: move |_| { nav.push(Route::ChatView { name: name.clone(), address: address.clone() }); },
                    {av}
                    div { style: "flex:1; min-width:0; display:flex; flex-direction:column; gap:5px;",
                        span {
                            style: format!("font-size:15px; color:{TEXT}; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;"),
                            "{name}"
                        }
                        span {
                            style: format!("font-size:13px; color:{SUB}; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;"),
                            "{preview}"
                        }
                    }
                    div { style: "display:flex; flex-direction:column; align-items:flex-end; gap:5px;",
                        span { style: format!("font-size:12px; color:{DIM};"), "{clock}" }
                        {status_el}
                    }
                }
            }
        })
        .collect();

    rsx! {
        if rows.is_empty() {
            div {
                style: format!("padding:28px 16px; text-align:center; font-size:13px; color:{DIM}; letter-spacing:2px; font-family:{MONO};"),
                if q.is_empty() { "NO CONTACTS YET" } else { "NO MATCHING CONTACTS" }
            }
        } else {
            for r in rows {
                {r}
            }
        }
    }
}
