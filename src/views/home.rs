use std::str::SplitWhitespace;

use crate::{
    DB, Route, WALLET,
    database::db_fns::{db_add_contact, db_read_contacts, db_store_init_message},
    views::DbContact,
    wallet::{utils::NODE_ENDPOINT, wallet_fns::wallet_get_seed},
};
use dioxus::{logger::tracing::info, prelude::*};

fn copy_feedback_color(is_empty: bool) -> &'static str {
    if is_empty { "transparent" } else { "#22c55e" }
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    // Use wl-copy directly — most reliable on Wayland
    use std::io::Write;
    
    // Check if wl-copy is available
    let output = std::process::Command::new("which")
        .arg("wl-copy")
        .output()
        .map_err(|_| "wl-copy not found".to_string())?;
    
    if !output.status.success() {
        // Fallback: try xclip for X11/Xwayland
        let output = std::process::Command::new("which")
            .arg("xclip")
            .output()
            .map_err(|_| "Neither wl-copy nor xclip found".to_string())?;
        
        if !output.status.success() {
            return Err("No clipboard tool available (need wl-copy or xclip)".to_string());
        }
        
        // Use xclip as fallback
        return std::process::Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
                child.wait()?;
                Ok(())
            })
            .map_err(|e| format!("xclip failed: {}", e));
    }
    
    // Use wl-copy
    std::process::Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
            child.wait()?;
            Ok(())
        })
        .map_err(|e| format!("wl-copy failed: {}", e))
}

#[component]
pub fn Home() -> Element {
    let nav = navigator();

    let mut contacts_vec = use_signal(|| vec![DbContact::default()]);

    let mut address = use_signal(|| String::new());
    let mut full_address = use_signal(|| String::new());
    let mut online_status = use_signal(|| String::new());
    let mut balance = use_signal(|| String::new());
    let mut topoheight = use_signal(|| 0);
    let mut sidebar = use_signal(|| String::from("none"));
    let mut copy_feedback = use_signal(|| String::new());

    // read contacts from db
    let mut db_contacts = use_resource(move || async move {
        if let Some(db) = &*DB.read() {
            db_read_contacts(db, &mut contacts_vec).await;
        }
    });

    // get state info
    use_future(move || async move {
        if let Some(wallet) = &*WALLET.read() {
            // get wallet address
            let local_address = wallet.read().await.get_address().await;
            *full_address.write() = local_address.clone();
            let address_len = local_address.len();

            // shorten it
            *address.write() = format!(
                "{}...{}",
                &local_address[..10],
                &local_address[address_len - 10..]
            )
            .to_string();

            //set wallet online
            match wallet
                .write()
                .await
                .set_online(NODE_ENDPOINT.to_string())
                .await
            {
                Ok(_) => online_status.set("Online".to_string()),
                Err(e) => {
                    if "Wallet is already in online mode" == e.to_string() {
                        info!("set_online error: {e}");
                        online_status.set("Online".to_string());
                    } else {
                        info!("set_online error: {e}");
                        online_status.set("Offline".to_string());
                    }
                }
            };

            // get balance
            if let Ok(ret_balance) = wallet.write().await.get_balance().await {
                balance.set(ret_balance);
            };

            // get topoheight
            topoheight.set(wallet.read().await.topoheight);

            // poll for new app events
            let mut refresh_db = false;
            loop {
                if let Some(wallet) = &*WALLET.read() {
                    wallet.write().await.backgroud_daemon().await;

                    // retrive the topoheight from the wallet
                    topoheight.set(wallet.read().await.topoheight);

                    // retrive the wallet balance from daemon updates
                    balance.set(wallet.read().await.balance.clone());

                    while let Some(tx) = wallet.write().await.rx_messages.pop() {
                        // store the message
                        db_store_init_message(tx).await;
                        refresh_db = true;
                    }

                    // reload the db
                    if refresh_db {
                        db_contacts.restart();
                        refresh_db = false
                    }
                }
            }
        }
    });
    rsx!(
        div {
            style: "position:absolute; background-color:#0a0a0a; color:#22c55e; width:320px; min-height:100vh; overflow-y:auto;",
            id: "sidebar",
            display: "{sidebar}",
            div {
                style: "padding-top:16px;",
                div {
                    style: "display:flex; justify-content:space-between;",
                    h1 {
                        style: "font-size:24px; font-weight:600; padding:16px;",
                        "XELITE"
                    }
                    button {
                        style: "padding:16px; font-size:24px; color:#22c55e;",
                        id: "open-sidebar",
                        onclick: move |_| sidebar.set("none".to_string()),
                        "<"
                    }
                }
                div {
                    style: "margin:16px; padding:8px 0; font-size:14px; color:#22c55e;",
                    if balance.read().is_empty() {
                        span { style: "color:#f59e0b;", "Syncing..." }
                    } else {
                        "xel: {balance.read()}"
                    }
                }
                ul {
                    style: "margin:16px;",
                    li {
                        style: "margin-bottom:8px; width:100%;",
                        button {
                            style: "display:block; color:#22c55e; width:100%; padding:8px; border:1px solid #22c55e; border-radius:8px; background-color:#0a0a0a;",
                            onclick: move |_| {
                                let addr = full_address.read().clone();
                                if addr.is_empty() {
                                    copy_feedback.set("Address not loaded yet".to_string());
                                    return;
                                }
                                match copy_to_clipboard(&addr) {
                                    Ok(_) => copy_feedback.set("✓ Copied!".to_string()),
                                    Err(e) => copy_feedback.set(format!("Error: {:?}", e)),
                                }
                            },
                            "📋 Copy Address"
                        }
                    }
                    li {
                        style: "margin-bottom:8px; width:100%; text-align:center; min-height:20px;",
                        color: "{copy_feedback_color(copy_feedback.read().is_empty())}",
                        "{copy_feedback.read()}"
                    }
                    li { style: "margin-bottom:8px;", button { style: "display:block; color:#22c55e;", onclick: move |_| {nav.push(Route::ViewSeed {});},"View Seed" } }
                    li { style: "margin-bottom:8px;", button { style: "display:block; color:#22c55e;", "Settings" } }
                    li { style: "margin-bottom:8px;", button { style: "display:block; color:#22c55e;", "Info" } }
                }
                div {
                    style: "position:absolute; bottom:0; left:0; margin:8px; font-size:12px; color:#166534;",
                    "Xelite v0.0.1"
                }
            }
        }

        div {
            style: "border:2px solid #16a34a; border-radius:12px; margin:16px;",
            div {
                style: "display:flex; justify-content:space-between; align-items:center;",
                button {
                    style: "font-size:20px; font-weight:600; color:#22c55e; padding:16px;",
                    id: "open-sidebar",
                    onclick: move |_| sidebar.set("block".to_string()),
                    ">"
                }
                h1 {
                    style: "font-size:20px; font-weight:600; color:#22c55e;",
                    if *online_status.read() == "Online" {
                        a { style: "" }
                    } else {
                        a { style: "" }
                    },
                    " {online_status.read()}"
                }
                h1 { style: "font-size:20px; font-weight:600; color:#22c55e;", " | " }
                h1 { style: "font-size:20px; font-weight:600; color:#22c55e;", "{topoheight.read()}" }
            }
        }

        // main
        div {
            style: "display:flex; flex-direction:column; padding:16px;",
            for contact in contacts_vec.read().iter().cloned() {
                // skip the default contact
                if DbContact::default() != contact {
                    button {
                        style: "border:2px solid #16a34a; border-radius:12px; padding:16px; margin-bottom:16px; color:#22c55e;",
                        onclick: move |_| { nav.push(Route::ChatView { name: contact.name.clone(), address: contact.address.clone() }); },
                        "{contact.name}"
                    }
                }
            }
        }

        div {
            style: "position:absolute; right:0; bottom:0; margin:32px;",
            button {
                style: "border:2px solid #6ee7b7; border-radius:12px; width:64px; height:64px; background-color:#22c55e; color:black; font-size:32px; font-weight:bold;",
                onclick: move |_| { nav.push(Route::AddContact {}); },
                "+"
            }
        }
    )
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
        // add the entry to the local database
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

    rsx!(
        div {
            style: "border:2px solid #16a34a; border-radius:12px; margin:16px;",
            div {
                style: "display:flex; justify-content:space-between; align-items:center;",
                button {
                    style: "font-size:20px; font-weight:600; color:#22c55e; padding:16px;",
                    id: "open-sidebar",
                    onclick: move |_| { nav.push(Route::Home {}); },
                    "<"
                }
                h1 {
                    style: "font-size:20px; font-weight:600; color:#22c55e;",
                    "Add Contact"
                }
                h1 {
                    style: "font-size:20px;",
                    ""
                }
            }
        }
    div {
        style: "height:100vh; display:flex; align-items:center; justify-content:center;",
        form {
            style: "padding:16px; width:100%; max-width:24rem;",
            onsubmit: add_contact,
            div {
                style: "display:flex;",
                input {
                    id: "contact-name",
                    style: "flex-grow: 1; border:2px solid #16a34a; border-radius:12px; padding:16px; margin-bottom:16px; color:#22c55e;",
                    placeholder: "Enter contact name...",
                    value: "{contact_name}",
                    oninput: move |event| contact_name.set(event.value())
                }
            }
            div {
                style: "display:flex;",
                input {
                    id: "contat-address",
                    style: "flex-grow: 1; border:2px solid #16a34a; border-radius:12px; padding:16px; margin-bottom:16px; color:#22c55e;",
                    placeholder: "Enter a wallet address...",
                    value: "{contact_address}",
                    oninput: move |event| contact_address.set(event.value())
                }
            }
            div {
                style: "display:flex; justify-content:center;",
                button {
                    style: "border:2px solid #6ee7b7; border-radius:12px; background-color:#22c55e; color:black; padding:16px;",
                    r#type: "submit",
                    "Add Contact"
                }
            }
            div {
                style: "color:#22c55e;",
                "{contact_ret_msg.read()}"
            }
        }
    }

    )
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

    let get_seed_phrase = move |_: FormEvent| async move {
        let entered_password = user_password.read().clone();

        match &*DB.read() {
            Some(db) => {
                wallet_get_seed(db, entered_password, &mut seed_phrase).await;
            }
            None => {
                info!("DB not accessible");
                seed_phrase.set("DB not accessible".to_string());
            }
        }
    };

    rsx!(
          div {
              style: "border:2px solid #16a34a; border-radius:12px; margin:16px;",
              div {
                  style: "display:flex; justify-content:space-between; align-items:center;",
                  button {
                      style: "font-size:20px; font-weight:600; color:#22c55e; padding:16px;",
                      id: "go-home",
                      onclick: move |_| { nav.push(Route::Home {}); },
                      "<"
                  }
                  h1 {
                      style: "font-size:20px; font-weight:600; color:#22c55e;",
                      "Get Seed Phrase"
                  }
                  h1 {
                      style: "font-size:20px;",
                      ""
                  }
              }
          }
      div {
          style: "margin:16px; display:grid; grid-template-columns:repeat(3, 1fr); gap:16px;",
          if seed_phrase().len() < 100 && !seed_phrase().is_empty() {
              div {
                  style: "grid-column:span 3; justify-self:center; border:2px solid #16a34a; border-radius:12px; padding:8px; margin-bottom:16px; color:#22c55e;",
                  "{seed_phrase()}"
              }
          }
          else {
               for (i, word) in seed_phrase().split_whitespace().enumerate() {
                   div {
                       style: "border:2px solid #16a34a; border-radius:12px; padding:8px; margin-bottom:16px; color:#22c55e;",
                       "{i}. {word}"
                   }
               }
          }
      }

      div {
          style: "height:100vh; display:flex; align-items:center; justify-content:center;",
          form {
               style: "padding:16px; width:100%; max-width:24rem;",
               onsubmit: get_seed_phrase,
               div {
                   style: "display:flex;",
                   input {
                         id: "account-password",
                         style: "flex-grow: 1; border:2px solid #16a34a; border-radius:12px; padding:16px; margin-bottom:16px; color:#22c55e;",
                         placeholder: "Enter account password...",
                         value: "{user_password}",
                         oninput: move |event| user_password.set(event.value())
                   }
               }
               div {
                   style: "display:flex; justify-content:center;",
                   button {
                          style: "border:2px solid #6ee7b7; border-radius:12px; background-color:#22c55e; color:black; padding:16px;",
                          r#type: "submit",
                          "Get Seed"
                   }
               }
           }
      }
    )
}
