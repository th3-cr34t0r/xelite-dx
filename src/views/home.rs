use crate::{
    DB, Route, WALLET,
    database::db_fns::{db_add_contact, db_read_contacts, db_store_init_message},
    views::DbContact,
    wallet::{utils::NODE_ENDPOINT, wallet_fns::wallet_get_seed},
};
use dioxus::{logger::tracing::info, prelude::*};

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
#[component]
pub fn Home() -> Element {
    let nav = navigator();

    let mut contacts_vec = use_signal(|| vec![DbContact::default()]);

    let mut address = use_signal(|| String::new());
    let mut online_status = use_signal(|| String::new());
    let mut balance = use_signal(|| String::new());
    let mut topoheight = use_signal(|| 0);
    let mut sidebar = use_signal(|| String::from("invisible"));

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
            *address.write() = wallet.read().await.get_address().await;
            let local_address = wallet.read().await.get_address().await;
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

                    // retrive the wallet balance
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
            class: "{sidebar} absolute bg-black text-green-600 outline-2 outline-green-700 w-80 min-h-screen overflow-y-auto transition-transform transform ease-in-out duration-300",
            id: "sidebar",
            div {
                class: "pt-4",
                div {
                    class: "flex justify-between",
                    h1 {
                        class: "text-2xl font-semibold p-4",
                        "XELITE"
                    }
                    button {
                        class: "p-4 text-xl text-green-600 hover:text-green-500",
                        id: "open-sidebar",
                        onclick: move |_| sidebar.set("hidden".to_string()),
                        "<"
                    }
                }
                ul {
                    class: "m-4",
                    li { class: "mb-2", button { class: "block hover:text-green-500", "Profile" } }
                    li { class: "mb-2", button { class: "block hover:text-green-500", "Settings" } }
                    li { class: "mb-2", button { class: "block hover:text-green-500", "Info" } }
                }
                div {
                    class: "absolute bottom-0 left-0 m-2 text-s text-green-900",
                    "Xelite v0.0.1"
                }
            }
        }

        div {
            class: "outline-2 outline-green-700 rounded-xl m-4",
            div {
                class: "container mx-auto",
                div {
                    class: "flex justify-between items-center ",
                    button {
                        class: "text-xl font-semibold text-green-600 hover:text-green-500 p-4",
                        id: "open-sidebar",
                        onclick: move |_| sidebar.set("".to_string()),
                        ">"
                    }
                    h1 {
                        class: "text-xl font-semibold text-green-600",
                        if *online_status.read() == "Online" {
                            a { class: "" }
                        } else {
                            a { class: "" }
                        },
                        " {online_status.read()}"
                    }
                    h1 { class: "text-xl font-semibold text-green-600", "|" }
                    h1 { class: "text-xl font-semibold text-green-600", "{topoheight.read()}" }
                    h1 { class: "", "" }
                }
            }
        }

        // main
        div {
            class: "flex flex-col p-4",
            for contact in contacts_vec.read().iter().cloned() {
                // skip the default contact
                if DbContact::default() != contact {
                    button {
                        class: "outline-2 outline-green-700 rounded-xl p-4 mb-4 text-green-600 hover:outline-green-500 hover:text-green-500",
                        onclick: move |_| { nav.push(Route::ChatView { name: contact.name.clone(), address: contact.address.clone() }); },
                        "{contact.name}"
                    }
                }
            }
        }

        div {
            class: "absolute right-0 bottom-0 m-8",
            button {
                class: "outline-2 outline-green-500 rounded-xl size-16 hover:bg-green-600 text-green-600 hover:text-black",
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
            class: "outline-2 outline-green-700 rounded-xl mx-4 mt-4",
            div {
                class: "container mx-auto",
                div {
                    class: "flex justify-between items-center ",
                    button {
                        class: "text-xl font-semibold text-green-600 hover:text-green-500 p-4",
                        id: "open-sidebar",
                        onclick: move |_| { nav.push(Route::Home {}); },
                        "<"
                    }
                    h1 {
                        class: "text-xl font-semibold text-green-600",
                        "Add Contact"
                    }
                    h1 {
                        class: "text-xl",
                        ""
                    }
                }
            }
        }
    div {
        class: "h-screen flex items-center justify-center",
        form {
            class: "p-4 w-full max-w-md",
            onsubmit: add_contact,
            div {
                class: "flex",
                input {
                    id: "contact-name",
                    class: "grow outline-2 outline-green-600 rounded-xl p-4 mb-4 text-green-600",
                    placeholder: "Enter contact name...",
                    value: "{contact_name}",
                    oninput: move |event| contact_name.set(event.value())
                }
            }
            div {
                class: "flex",
                input {
                    id: "contat-address",
                    class: "grow outline-2 outline-green-600 rounded-xl p-4 mb-4 text-green-600",
                    placeholder: "Enter a wallet address...",
                    value: "{contact_address}",
                    oninput: move |event| contact_address.set(event.value())
                }
            }
            div {
                class: "flex justify-center",
                button {
                    class: "outline-2 outline-green-500 rounded-xl hover:bg-green-600 text-green-600 hover:text-black p-4",
                    r#type: "submit",
                    "Add Contact"
                }
            }
            div {
                class: "text-green-600",
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
    let mut seed_phrase_info = use_signal(|| String::new());

    let get_seed_phrase = move |_: FormEvent| async move {
        let entered_password = user_password.read().clone();

        match &*DB.read() {
            Some(db) => wallet_get_seed(db, entered_password, &mut seed_phrase_info).await,
            None => {
                info!("DB not accessible");
                seed_phrase_info.set("DB not accessible".to_string());
            }
        }
    };

    rsx!(
        div { button { onclick: move |_| {nav.push(Route::Home {});}, "Back"}}
        div { "SEED: {seed_phrase_info.read()}" }

        form {
              class: "provide_seed",
              onsubmit: get_seed_phrase,

          div {
                  input {
                      id: "password",
                      placeholder: "Enter account password...",
                      value: "{user_password}",
                      oninput: move |event| user_password.set(event.value())
                  }

                    button { r#type: "submit", "Get Seed" }
              }
          }
    )
}
