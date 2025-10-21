use crate::{
    DB, IS_READY, Route, WALLET,
    database::db_fns::{
        db_read_messages, db_remove_contact, db_store_init_message, db_update_status_topoheight,
    },
    views::DbMessage,
    wallet::wallet_fns::{DEV_FEE_AMOUNT, wallet_send_message},
};
use chrono::Utc;
use chrono::{self, TimeZone};
use dioxus::{logger::tracing::info, prelude::*};

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
#[component]
pub fn ChatView(name: String, address: String) -> Element {
    let nav = navigator();

    let timestamp = Utc::now().format("%H:%M %d %m %Y").to_string();

    let contact_name = use_signal(|| name);
    let contact_address = use_signal(|| address);
    let mut topoheight = use_signal(|| 0);
    let mut last_msg_fee = use_signal(|| 0.0);

    let mut messages_from_db = use_signal(|| vec![DbMessage::default()]);

    // load the messages from the database
    let mut db_message_handle = use_future(move || async move {
        let address = contact_address.read().clone();

        match &*DB.read() {
            Some(db) => db_read_messages(db, address, &mut messages_from_db).await,
            None => {
                info!("Error reading DB");
            }
        };

        // get the last message fee
        if let Some(message) = messages_from_db.read().last() {
            last_msg_fee.set(message.fee + DEV_FEE_AMOUNT);
        }
    });

    // message signal
    let mut send_msg = use_signal(|| String::new());
    let mut info = use_signal(|| String::new());
    let mut wallet_is_ready = use_signal(|| true);

    let subbmit_tx_message = move |_: FormEvent| async move {
        // only store input msg if it is not empty
        if !(*send_msg.read()).is_empty() {
            let message = send_msg.read().clone();
            send_msg.set("".to_string());

            *IS_READY.write().write().await = false;
            wallet_is_ready.set(false);

            wallet_send_message(
                contact_address.read().clone(),
                *topoheight.read(),
                message,
                &mut db_message_handle,
                &mut info,
            )
            .await;
        }
    };

    let remove_contact = move |_: FormEvent| async move {
        let address = contact_address.read().clone();

        match &*DB.read() {
            Some(db) => db_remove_contact(db, address).await,
            None => info!("Error reading DB"),
        }
    };

    // pool for new app events
    use_future(move || async move {
        loop {
            if let Some(wallet) = &*WALLET.read() {
                // retrive the topoheight from the wallet
                topoheight.set(wallet.read().await.topoheight);

                // check for app events
                wallet.write().await.backgroud_daemon().await;

                // store received messages in db
                while let Some(tx) = wallet.write().await.rx_messages.pop() {
                    // store the message
                    db_store_init_message(tx).await;

                    // reload the db
                    db_message_handle.restart();
                }

                while let Some(message) = wallet.write().await.confirmed_messages.pop() {
                    db_update_status_topoheight(message).await;

                    *IS_READY.write().write().await = true;
                    wallet_is_ready.set(true);
                    db_message_handle.restart();
                }
            }
        }
    });

    rsx!(
        div {class:"flex flex-col ",
                        header { class:"outline-2 outline-green-700 rounded-xl m-4",
                                div { class:"container mx-auto",
                                    div { class:"flex justify-between items-center ",
                                    button { class:"text-xl font-semibold text-green-600 hover:text-green-500 p-4", id: "open-sidebar", onclick: move |_| {nav.push(Route::Home {});}, "<"}

                            h1 { class:"text-xl font-semibold text-green-600", "{contact_name()}" }

                            h1 { class:"text-xl text-green-600", "{topoheight()}" }
                                        }
                            }
                    }
                div { class:"text-green-600", "{contact_address()}" }
                div { class:"text-green-600", "Last message fee: {last_msg_fee()}" }

                div {
                      form {
                        onsubmit: remove_contact,
                        button {class:"text-green-600 hover:text-green-500", r#type:"submit", "Delete Contact" }
                      }
                }

    main {class:"flex-1 overflow-auto outline-2 outline-green-600 rounded-xl m-4 h-full",
                    for msg in messages_from_db.cloned().iter() {
                        if msg.message.is_some() {
                            if msg.direction == "Outgoing" {
                                div { class:"",
                                    a { class:"text-green-900 mx-2", "Topoheight: {msg.topoheight}" }
                                    a { class:"text-green-900 mx-2", "{msg.status}" }
                                    a { class:"text-green-600", "> {msg.message.as_ref().unwrap()}" }
                                }
                            }
                            else {
                                div { class:"bg-green-600",
                                    div { class:"bg-green-600", a { class:"", "Topoheight: {msg.topoheight}" } }
                                    div { class:"bg-green-600",
                                        "{msg.message.as_ref().unwrap()}"
                                    }
                                    div { class:"bg-green-600", "{msg.status}" }
                                }
                            }

                        }
                    }
    }
    footer { class:"fixed left-0 right-0 bottom-0 m-4",
                    form {
                        onsubmit: move |event| async move{
                            event.prevent_default();
                            subbmit_tx_message(event).await;
                        },
                        div { class:"flex",
                          input {class:"outline-2 outline-green-600 rounded-xl p-4 text-green-600", oninput: move |event| send_msg.set(event.value()), class:"",value:"{send_msg}",id:"message-input", placeholder:"> type a secure message...", autofocus: true },
                          button {class:"outline-2 outline-green-600 rounded-xl mx-4 p-4 text-green-600 hover:text-green-500", disabled: !wallet_is_ready(), r#type:"submit", "Send"}
                        }
                    }
                div { a {class:"bg-green-600", "{info()}" } }

            }
        }
        )
}
