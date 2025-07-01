use crate::{
    views::{DbAddress, DbMessage, DbRemoveContact, MessageDirection, RxMsg, TxSendMsg},
    Route, DB,
};
use chrono::Utc;
use dioxus::{logger::tracing::info, prelude::*};
use sqlx::query;

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

    let mut msgs_vec = use_signal(|| vec![DbMessage::default()]);

    // load the messages from the database
    let mut load_database = use_future(move || async move {
        let address = contact_address.read().clone();

        // let args = serde_wasm_bindgen::to_value(&DbAddress { address }).unwrap();

        // let ret_msgs_vec = invoke("db_read_chat_messages", args.clone()).await;

        // let db_msgs_vec: Result<Vec<DbMessage>, Error> =
        //     serde_wasm_bindgen::from_value(ret_msgs_vec);

        // if let Ok(db_msgs_vec) = db_msgs_vec {
        //     // clear the vec before populating it again
        //     msgs_vec.clear();
        //     for message in db_msgs_vec.iter() {
        //         // compare the timestamps and order them old -> new
        //         // right now only display messages for the given address (contact)
        //         msgs_vec.push(message.clone());
        //     }
        // }
    });

    let mut topoheight_event = use_signal(|| u64::MIN);
    // pool for topoheight
    use_future(move || async move {
        // let mut listener = listen::<u64>("new_topoheight_event").await.unwrap();

        // while let Some(event) = listener.next().await {
        //     let Event {
        //         event: _,
        //         id: _,
        //         payload,
        //     } = event;
        //     // store the new value
        //     topoheight_event.set(payload);
        // }
    });

    // message signal
    let mut msg_to_tx = use_signal(|| String::new());
    let mut submit_tx_message_info = use_signal(|| String::new());

    let subbmit_tx_message = move |_: FormEvent| async move {
        // only store input msg if it is not empty
        if !(*msg_to_tx.read()).is_empty() {
            // todo: prepare a tx and send it
            // todo: store the successfully sent tx localy in the database

            //     let address = contact_address.read().clone();
            //     let direction = MessageDirection::Outgoing;
            //     let topoheight = topoheight_event() as i64;
            //     let timestamp = timestamp;
            //     let message = msg_to_tx.read().clone();

            //     // send the msg tx to the contact address
            //     let tx_args = serde_wasm_bindgen::to_value(&TxSendMsg {
            //         message: message.clone(),
            //         address: address.clone(),
            //     })
            //     .unwrap();

            //     if let Some(tx_ret_val) = invoke("send_tx_msg", tx_args).await.as_string() {
            //         if tx_ret_val == "Ok".to_string() {
            //             // store message in db
            //             let db_args = serde_wasm_bindgen::to_value(&DbMessage {
            //                 address,
            //                 direction,
            //                 topoheight,
            //                 timestamp,
            //                 message,
            //             })
            //             .unwrap();

            //             if let Some(ret_val) = invoke("db_store_chat_messages", db_args.clone())
            //                 .await
            //                 .as_string()
            //             {
            //                 submit_tx_message_info.set(ret_val);
            //             };

            //             // read database
            //             load_database.restart();
            //         }
            //         msg_to_tx.set("".to_string());
            //     }
        }
    };

    let mut rx_msg_info = use_signal(|| vec![RxMsg::default()]);
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
                    match query("DELETE FROM messages WHERE address = ?1")
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

    rsx!(
        div { button { onclick: move |_| {nav.push(Route::Home {});}, "Back"}}
        div { a { "{contact_name()}" } }
        div { a { "{contact_address()}" } }
        div { a { "{remove_contact_info()}" } }
        div { a { "Topoheight: {topoheight_event()}" } }

        for msg in rx_msg_info().iter(){

            div { a { "Received from: {msg.from_address:?}" } }
            div { a { "Received message: {msg.transfer.message:?}" } }
        }

        div {
              form {
                onsubmit: remove_contact,
                button { r#type:"submit", "Delete Contact" }
              }
        }
        div {
            // display outgoing msgs
            for msg in msgs_vec.iter() {
                if MessageDirection::Err != msg.direction{
                    div {
                         a { b { "{msg.direction:?}({msg.topoheight}): {msg.message}" } }
                    }
                }
            }
            div { a {"{submit_tx_message_info.read()}"} }

            form {
                onsubmit: subbmit_tx_message,
                input { oninput: move |event| msg_to_tx.set(event.value()), value:"{msg_to_tx}",id:"message-input", placeholder:"Message...", autofocus: true }
                button { r#type:"submit", "Send"}
            }
        }
    )
}
