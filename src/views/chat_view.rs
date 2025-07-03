use crate::{
    database::db_fns::{db_read_messages, db_remove_contact, db_store_message},
    views::DbMessage,
    wallet::wallet_fns::wallet_send_message,
    Route, DB, WALLET,
};
use chrono::Utc;
use dioxus::{logger::tracing::info, prelude::*};

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
#[component]
pub fn ChatView(name: String, address: String) -> Element {
    let nav = navigator();

    let timestamp = Utc::now().timestamp();

    let contact_name = use_signal(|| name);
    let contact_address = use_signal(|| address);
    let mut topoheight = use_signal(|| 0);

    let mut messages_from_db = use_signal(|| vec![DbMessage::default()]);

    // load the messages from the database
    let mut db_message_handle = use_future(move || async move {
        let address = contact_address.read().clone();

        match &*DB.read() {
            Some(db) => db_read_messages(db, address, &mut messages_from_db).await,
            None => {
                info!("Error reading DB");
            }
        }
    });

    // message signal
    let mut msg_to_send = use_signal(|| String::new());
    let mut info = use_signal(|| String::new());
    let subbmit_tx_message = move |_: FormEvent| async move {
        // only store input msg if it is not empty
        if !(*msg_to_send.read()).is_empty() {
            wallet_send_message(
                contact_address.read().clone(),
                timestamp,
                *topoheight.read(),
                &mut msg_to_send,
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
        let mut refresh_db = false;
        loop {
            if let Some(wallet) = &*WALLET.read() {
                // retrive the topoheight from the wallet
                topoheight.set(wallet.read().await.topoheight);

                // check for app events
                wallet.write().await.backgroud_daemon().await;

                // store received messages in db
                while let Some(tx) = wallet.write().await.rx_messages.pop() {
                    // store the message
                    db_store_message(tx).await;
                    refresh_db = true;
                }

                // reload the db
                if refresh_db {
                    db_message_handle.restart();
                    refresh_db = false
                }
            }
        }
    });

    rsx!(
        div { button { onclick: move |_| {nav.push(Route::Home {});}, "Back"}}
        div { a { "{contact_name()}" } }
        div { a { "{contact_address()}" } }
        div { a { "Topoheight: {topoheight()}" } }

        div {
              form {
                onsubmit: remove_contact,
                button { r#type:"submit", "Delete Contact" }
              }
        }
        div {
            // display outgoing msgs
            for msg in messages_from_db.cloned().iter() {
                if msg.message.is_some() {
                    div {
                         a { b { "{msg.direction}({msg.topoheight}): {msg.message.as_ref().unwrap()}" } }
                    }

                }
            }

            form {
                onsubmit: subbmit_tx_message,
                input { oninput: move |event| msg_to_send.set(event.value()), value:"{msg_to_send}",id:"message-input", placeholder:"Message...", autofocus: true }
                button { r#type:"submit", "Send"}
            }
        }
        div { a { "{info()}" } }
    )
}
