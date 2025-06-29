use crate::{
    views::{DbContact, DbPassword},
    wallet::utils::NODE_ENDPOINT,
    Route, WALLET,
};
use dioxus::{logger::tracing::info, prelude::*};
// use dioxus_logger::tracing::info;

#[component]
pub fn Home() -> Element {
    let nav = navigator();

    let mut address = use_signal(|| String::new());
    // gets the wallet address
    use_future(move || async move {
        if let Some(wallet) = &*WALLET.read() {
            *address.write() = wallet.read().await.get_address().await;
        }
    });

    let mut online_status = use_signal(|| String::new());
    // gets the wallet connection status
    use_future(move || async move {
        info!("Wallet online_status start");
        if let Some(wallet) = &*WALLET.read() {
            if let Ok(_) = wallet
                .write()
                .await
                .set_online(NODE_ENDPOINT.to_string())
                .await
            {
                online_status.set("online".to_string());
            } else {
                online_status.set("offline".to_string());
            };
        }
        info!("Wallet online_status end");
    });

    let mut balance = use_signal(|| String::new());
    // gets the wallet balance
    use_future(move || async move {
        if let Some(wallet) = &*WALLET.read() {
            if let Ok(ret_balance) = wallet.read().await.get_balance().await {
                balance.set(ret_balance);
            };
        }
    });

    let mut contacts_vec = use_signal(|| {
        vec![DbContact {
            name: "".to_string(),
            address: "".to_string(),
        }]
    });
    // read contacts from db
    use_future(move || async move {
        // let ret_val = invoke("db_read_contacts", JsValue::null()).await;

        // let rx_contacts_vec: Vec<DbContact> = serde_wasm_bindgen::from_value(ret_val).unwrap();

        // for contact in rx_contacts_vec.iter() {
        //     contacts_vec.push(CopyValue::new(contact.clone()));
        // }
    });

    let mut topoheight = use_signal(|| u64::MIN);
    // pool for new app events
    use_future(move || async move {
        // let mut listener = listen::<u64>("new_topoheight_event").await.unwrap();

        // while let Some(event) = listener.next().await {
        //     let Event {
        //         event: _,
        //         id: _,
        //         payload,
        //     } = event;
        //     // store the new value
        //     app_events.set(payload);
        // }
    });
    rsx!(
        div { "Home Screen" }
        div { "Wallet Address: {address.read()}"}
        div { "Balance: {balance.read()} XEL"}
        div { "Status: {online_status.read()}"}
        div { a { "Topoheight: {topoheight.read()}" } }
        div {button {onclick: move |_| {nav.push(Route::AddContact {});},"Add Contact"}}
        div {button {onclick: move |_| {nav.push(Route::ViewSeed {});},"View Seed Phrase"}}

        div {
            for contact in contacts_vec().iter().cloned() {
                div {
                    button { onclick:  move |_|  {nav.push(Route::ChatView {name: contact.name.clone(), address: contact.address.clone()});}, "{contact.name}"}
                }
            }
        }
    )
}

#[component]
pub fn AddContact() -> Element {
    let nav = navigator();

    let mut contact_address = use_signal(|| String::new());
    let mut contact_name = use_signal(|| String::new());
    let mut contact_ret_msg = use_signal(|| String::new());

    let add_contact = move |_: FormEvent| async move {
        // add the entry to the local database
        let name = contact_name.read().clone();
        let address = contact_address.read().clone();

        // if !name.is_empty() && !address.is_empty() {
        //     let args = serde_wasm_bindgen::to_value(&DbContact { address, name }).unwrap();

        //     if let Some(ret_info) = invoke("db_add_contact", args.clone()).await.as_string() {
        //         contact_ret_msg.set(ret_info);
        //     }
        // } else {
        //     contact_ret_msg.set("Name and address cannot be empty".to_string());
        // }
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

#[component]
pub fn ViewSeed() -> Element {
    let nav = navigator();

    let mut user_password = use_signal(|| String::new());
    let mut seed_phrase_info = use_signal(|| String::new());

    let get_seed_phrase = move |_: FormEvent| async move {
        let entered_password = user_password.read().clone();

        // let args = serde_wasm_bindgen::to_value(&DbPassword {
        //     password: entered_password,
        // })
        // .unwrap();

        // if let Some(seed_phrase) = invoke("get_seed_phrase", args.clone()).await.as_string() {
        //     seed_phrase_info.set(seed_phrase);
        // } else {
        //     seed_phrase_info.set("Error".to_string());
        // }
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
