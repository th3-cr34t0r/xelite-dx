use crate::{
    database::sql::db_store_message, views::DbMessage, wallet::utils::Transfer, Route, DB, WALLET,
};
use chrono::Utc;
use dioxus::{logger::tracing::info, prelude::*};
use futures::TryStreamExt;
use sqlx::{query, query_as, Error};
use xelis_common::config::XELIS_ASSET;

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
    let mut topoheight = use_signal(|| u64::MIN);

    let mut messages_from_db = use_signal(|| vec![DbMessage::default()]);

    // load the messages from the database
    let mut message_db = use_future(move || async move {
        let address = contact_address.read().clone();

        match &*DB.read() {
            Some(db) => {
                let db_messages: Result<Vec<DbMessage>, Error> = query_as(
                 "SELECT direction, address, hash, timestamp, topoheight, asset, amount, message FROM Message WHERE address = ?",
             )
             .bind(address)
             .fetch_all(&*db)
             .await;

                match db_messages {
                    Ok(db_messages) => {
                        // empty the vec
                        messages_from_db().clear();

                        //assign the new db value
                        *messages_from_db.write() = db_messages;
                    }
                    Err(e) => {
                        info!("{}", e);
                    }
                }
            }
            None => {
                info!("Error reading DB");
            }
        }
    });

    // message signal
    let mut msg_to_send = use_signal(|| String::new());
    let mut submit_tx_message_info = use_signal(|| String::new());

    let subbmit_tx_message = move |_: FormEvent| async move {
        // only store input msg if it is not empty
        if !(*msg_to_send.read()).is_empty() {
            // todo: prepare a tx and send it
            // todo: store the successfully sent tx localy in the database

            let mut message = DbMessage {
                direction: "Outgoing".to_string(),
                address: contact_address.read().clone(),
                hash: Default::default(),
                timestamp,
                topoheight: topoheight() as i64,
                asset: XELIS_ASSET.to_string(),
                amount: Default::default(),
                message: Some(msg_to_send.read().clone()),
            };

            match &*WALLET.read() {
                Some(wallet_rw) => {
                    let mut wallet = wallet_rw.write().await;

                    // create the vector of transfers
                    let transfers = vec![Transfer {
                        float_amount: 0.0,
                        str_address: contact_address.read().clone(),
                        asset_hash: XELIS_ASSET.to_string(),
                        extra_data: message.message.clone(),
                    }];

                    let transaction_summary = wallet
                        .create_transfers_transaction(transfers)
                        .await
                        .unwrap();

                    match wallet
                        .broadcast_transaction(transaction_summary.hash.clone())
                        .await
                    {
                        Ok(_) => {
                            info!("Message sent successfully");
                            message.hash = transaction_summary.hash;
                            // store it in db
                            db_store_message(message).await;

                            // reload database
                            message_db.restart();

                            // reset message field
                            msg_to_send.set("".to_string());
                        }
                        Err(e) => {
                            info!("{}", e)
                        }
                    }
                }
                None => {
                    info!("Error reading wallet");
                }
            }
        }
    };

    let mut rx_msg_info = use_signal(|| vec![DbMessage::default()]);
    use_future(move || async move {
        // poll the wallet for new inputs

        // let mut listener = listen::<RxMsg>("new_transaction_event").await.unwrap();

        // while let Some(event) = listener.next().await {
        //     let Event {
        //         event: _,
        //         id: _,
        //         payload,
        //     } = event;

        //     topoheight_event.set(payload.topoheight);
        //     rx_msg_info.write().push(payload);
        //     // rerun loading database
        //     load_database.restart();
        // }
    });

    let mut remove_contact_info = use_signal(|| String::new());
    let remove_contact =
        move |_: FormEvent| async move {
            let address = contact_address.read().clone();

            match &*DB.read() {
                Some(db) => {
                    match query("DELETE FROM Message WHERE address = ?1")
                        .bind(address.clone())
                        .execute(&*db)
                        .await
                    {
                        Ok(_) => {
                            match query("DELETE FROM contacts WHERE address = ?1")
                                .bind(address)
                                .execute(&*db)
                                .await
                            {
                                Ok(_) => remove_contact_info
                                    .set("Contact removed successfully".to_string()),
                                Err(e) => remove_contact_info.set(e.to_string()),
                            }
                        }
                        Err(e) => {
                            info!("{e}");
                            match query("DELETE FROM contacts WHERE address = ?1")
                                .bind(address)
                                .execute(&*db)
                                .await
                            {
                                Ok(_) => remove_contact_info
                                    .set("Contact removed successfully".to_string()),
                                Err(e) => remove_contact_info.set(e.to_string()),
                            }
                        }
                    }
                }
                None => {
                    remove_contact_info.set("Error reading DB".to_string());
                }
            }
        };

    // pool for new app events
    use_future(move || async move {
        let mut refresh_db = false;
        loop {
            if let Some(wallet) = &*WALLET.read() {
                wallet.write().await.backgroud_daemon().await;

                // retrive the topoheight from the wallet
                topoheight.set(wallet.read().await.topoheight);

                while let Some(tx) = wallet.write().await.rx_messages.pop() {
                    // store the message
                    db_store_message(tx).await;
                    refresh_db = true;
                }

                // reload the db
                if refresh_db {
                    message_db.restart();
                    refresh_db = false
                }
            }
        }
    });

    rsx!(
        div { button { onclick: move |_| {nav.push(Route::Home {});}, "Back"}}
        div { a { "{contact_name()}" } }
        div { a { "{contact_address()}" } }
        div { a { "{remove_contact_info()}" } }
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
            div { a {"{submit_tx_message_info.read()}"} }

            form {
                onsubmit: subbmit_tx_message,
                input { oninput: move |event| msg_to_send.set(event.value()), value:"{msg_to_send}",id:"message-input", placeholder:"Message...", autofocus: true }
                button { r#type:"submit", "Send"}
            }
        }
    )
}
