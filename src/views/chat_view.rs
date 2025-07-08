use crate::{
    database::db_fns::{
        db_read_messages, db_remove_contact, db_store_init_message, db_update_status_topoheight,
    },
    views::DbMessage,
    wallet::wallet_fns::{wallet_send_message, DEV_FEE_AMOUNT},
    Route, DB, IS_READY, WALLET,
};
use chrono::Utc;
use chrono::{self, TimeZone};
use daisy_rsx::{Button, ButtonSize, ButtonStyle, ButtonType};
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
        div { button { onclick: move |_| {nav.push(Route::Home {});}, Button {class: "btn-neutral", popover_target: "set-name-drawer", button_style: ButtonStyle::Outline, "Back"} }}
        div { a { "{contact_name()}" } }
        div { a { "{contact_address()}" } }
        div { a { "Topoheight: {topoheight()}" } }
        div { a { "Last message fee: {last_msg_fee()}" } }

        div {
              form {
                onsubmit: remove_contact,
                button { r#type:"submit", "Delete Contact" }
              }
        }

            for msg in messages_from_db.cloned().iter() {
                if msg.message.is_some() {
                    if msg.direction == "Outgoing" {
                        div { class:"chat chat-end",
                            div { class:"chat-header", a { class:"text-xs opacity-50", "Topoheight: {msg.topoheight}" } }
                            div { class:"chat-bubble",
                                "{msg.message.as_ref().unwrap()}"
                            }
                            div { class:"chat-footer opacity-50", "{msg.status}" }
                        }
                    }
                    else {
                        div { class:"chat chat-start",
                            div { class:"chat-header", a { class:"text-xs opacity-50", "Topoheight: {msg.topoheight}" } }
                            div { class:"chat-bubble",
                                "{msg.message.as_ref().unwrap()}"
                            }
                            div { class:"chat-footer opacity-50", "{msg.status}" }
                        }
                    }

                }
            }

        div {
            form {
                onsubmit: move |event| async move{
                    event.prevent_default();
                    subbmit_tx_message(event).await;
                },
            div { class:"flex flex-row gap-20",
                  input { oninput: move |event| send_msg.set(event.value()), class:"input grow gap-4",value:"{send_msg}",id:"message-input", placeholder:"Message...", autofocus: true },
                  button {class:"btn btn-primary w-14", disabled: !wallet_is_ready(), r#type:"submit", "Send"}
                }
            }
        }
        div { a { "{info()}" } }
    )
}
