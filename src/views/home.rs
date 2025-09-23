use crate::{
    database::db_fns::{db_add_contact, db_read_contacts, db_store_init_message},
    views::DbContact,
    wallet::{utils::NODE_ENDPOINT, wallet_fns::wallet_get_seed},
    Route, DB, WALLET,
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
            // let local_address = wallet.read().await.get_address().await;
            // let address_len = local_address.len();

            // shorten it
            // *address.write() = format!(
            //     "{}...{}",
            //     &local_address[..10],
            //     &local_address[address_len - 10..]
            // )
            // .to_string();

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

            div { class:"navbar",
                div { class:"navbar-start",
                    div { class:"dropdown",
                        div {"tabIndex":"0", role:"button", class:"btn",
                            button { class:"btn"}
                        }
                        ul { "tabIndex":"0", class:"",
                            li {
                                a {class:"text-xl", "{address.read()}"}
                            }
                            li {
                                link {class:"",onclick: move |_| {nav.push(Route::ViewSeed {});},"View Seed Phrase"}
                            }
                            li { a { "Hompeage" } }
                        }
                    }
                }
                div { class:"navbar-center",
                    div { class:"", if *online_status.read() == "Online" { a {class:""} } else { a { class:"" } },  " {online_status.read()}" }
                    div { class:"", a {class:"", " | {topoheight.read()}"} }
                }
                div { class:"navbar-end"}
            } // nav end

            main { class:"flex-grow p-4 h-screen overflow-auto",

                div {class:"justify-start",
                        for contact in contacts_vec.read().iter().cloned() {
                            // skip the default contact
                            if DbContact::default() != contact {
                    div {class:"card",
                                link { class:"card-body", onclick:  move |_|  {nav.push(Route::ChatView {name: contact.name.clone(), address: contact.address.clone()});},
                                    div {class:"flex",
                                        div { class:"bg-neutral text-neutral-content size-24 rounded-full", span { class:"text-3xl", "{&contact.name[..1]}" } }
                                        div { class:"flex-1 card-title", "{contact.name}"}
                                    }
                                }

                            }
                        }
                    }
                    div {class:"justify-items-end",button {class:"btn btn-circle btn-soft btn-accent", onclick: move |_| {nav.push(Route::AddContact {});}, "+" }}
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
    div { button { onclick: move |_| {nav.push(Route::Home {});}, "Back"}}
          form {
                class: "row",
                onsubmit: add_contact,

            div {
                    input {
                        id: "contact-name",
                        placeholder: "Enter contact name...",
                        value: "{contact_name}",
                        oninput: move |event| contact_name.set(event.value())
                    }
                }
            div {
                    input {
                        id: "contat-address",
                        placeholder: "Enter a wallet address...",
                        value: "{contact_address}",
                        oninput: move |event| contact_address.set(event.value())
                    },
                }
            div {
                    button { r#type: "submit", "Add Contact" }
                }

            div {
                a {"{contact_ret_msg.read()}"}
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
