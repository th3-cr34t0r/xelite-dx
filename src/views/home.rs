use crate::{
    database::sql::DbUserLogin, views::DbContact, wallet::utils::NODE_ENDPOINT, Route, DB, WALLET,
};
use dioxus::{logger::tracing::info, prelude::*};
use futures::TryStreamExt;
use sqlx::{query, query_as, Error};

#[allow(
    clippy::redundant_closure,
    clippy::await_holding_invalid_type,
    clippy::borrow_deref_ref
)]
#[component]
pub fn Home() -> Element {
    let nav = navigator();

    // gets the wallet address
    let mut address = use_signal(|| String::new());
    use_future(move || async move {
        if let Some(wallet) = &*WALLET.read() {
            *address.write() = wallet.read().await.get_address().await;
        }
    });

    //set wallet online
    let mut online_status = use_signal(|| String::new());
    use_future(move || async move {
        if let Some(wallet) = &*WALLET.read() {
            match wallet
                .write()
                .await
                .set_online(NODE_ENDPOINT.to_string())
                .await
            {
                Ok(_) => online_status.set("online".to_string()),
                Err(e) => {
                    if "Wallet is already in online mode" == e.to_string() {
                        info!("set_online error: {e}");
                        online_status.set("online".to_string());
                    } else {
                        info!("set_online error: {e}");
                        online_status.set("offline".to_string());
                    }
                }
            };
        }
    });

    let mut balance = use_signal(|| String::new());
    let mut topoheight = use_signal(|| u64::MIN);
    // gets the wallet info
    use_future(move || async move {
        if let Some(wallet) = &*WALLET.read() {
            if let Ok(ret_balance) = wallet.write().await.get_balance().await {
                balance.set(ret_balance);
            };

            topoheight.set(wallet.read().await.topoheight);
        }
    });

    let mut contacts_vec = use_signal(|| vec![DbContact::default()]);
    // read contacts from db
    use_future(move || async move {
        if let Some(db) = &*DB.read() {
            // create base table if it does not exist
            query(
                "CREATE TABLE IF NOT EXISTS contacts ( name TEXT NOT NULL, address TEXT NOT NULL )",
            )
            .execute(&*db)
            .await
            .expect("Cannot create contacts DB");

            let db_contacts: Result<Vec<DbContact>, Error> =
                query_as("SELECT name, address FROM contacts")
                    .fetch_all(&*db)
                    .await;

            match db_contacts {
                Ok(mut db_contacts) => {
                    while let Some(contact_from_db) = db_contacts.pop() {
                        info!("{contact_from_db:?}");
                        contacts_vec.write().push(DbContact {
                            name: contact_from_db.name,
                            address: contact_from_db.address,
                        });
                    }
                }
                Err(e) => {
                    info!("DbContacts retrived with error {e}");
                }
            }
        }
    });

    // pool for new app events
    use_future(move || async move {
        loop {
            if let Some(wallet) = &*WALLET.read() {
                wallet.write().await.backgroud_daemon().await;

                // retrive the topoheight from the wallet
                topoheight.set(wallet.read().await.topoheight);

                // retrive the wallet balance
                balance.set(wallet.read().await.balance.clone());
            }
        }
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
            for contact in contacts_vec.read().iter().cloned() {
                // skip the default contact
                if DbContact::default() != contact {
                    div {
                        button { onclick:  move |_|  {nav.push(Route::ChatView {name: contact.name.clone(), address: contact.address.clone()});}, "{contact.name}"}
                    }
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
pub fn AddContact() -> Element {
    let nav = navigator();

    let mut contact_address = use_signal(|| String::new());
    let mut contact_name = use_signal(|| String::new());
    let mut contact_ret_msg = use_signal(|| String::new());

    let add_contact = move |_: FormEvent| async move {
        // add the entry to the local database
        let name = contact_name.read().clone();
        let address = contact_address.read().clone();

        if !name.is_empty() && !address.is_empty() {
            if let Some(db) = &*DB.read() {
                // create base table if it does not exist
                query(
                "CREATE TABLE IF NOT EXISTS contacts ( name TEXT NOT NULL, address TEXT NOT NULL )",
            )
            .execute(&*db)
            .await
            .expect("Cannot create contacts DB");

                let all_contacts: Result<Vec<DbContact>, Error> =
                    query_as("SELECT * FROM contacts")
                        .fetch(&*db)
                        .try_collect()
                        .await;

                match all_contacts {
                    Ok(all_contacts_vec) => {
                        let ref_contact = DbContact {
                            name: name.clone(),
                            address: address.clone(),
                        };

                        let mut is_contained = false;

                        // check if the name and address is already contained
                        for contact in all_contacts_vec.iter() {
                            if ref_contact == *contact {
                                is_contained = true;
                            }
                        }

                        if !is_contained {
                            match query("INSERT INTO contacts (name, address) VALUES (?1, ?2)")
                                .bind(name.as_str())
                                .bind(address.as_str())
                                .execute(&*db)
                                .await
                            {
                                Ok(_) => {
                                    info!("Contact successfully added");
                                    contact_ret_msg.set("Contact successfully added".to_string());
                                }
                                Err(e) => {
                                    info!("Error adding contact to db: {e}");
                                    contact_ret_msg.set(
                                        format!("Error adding contact to db: {}", e).to_string(),
                                    );
                                }
                            }
                        } else {
                            info!("Contact already exists");

                            contact_ret_msg.set("Contact already exists".to_string());
                        }
                    }
                    Err(e) => {
                        info!("{e}");
                        contact_ret_msg.set(e.to_string());
                    }
                }
            }
        } else {
            contact_ret_msg.set("Name and address cannot be empty".to_string());
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
            Some(db) => {
                let db_user: Result<DbUserLogin, Error> =
                    query_as("SELECT username, password FROM user")
                        .fetch_one(&*db)
                        .await;

                match db_user {
                    Ok(db_user) => {
                        if entered_password == db_user.password {
                            if let Some(wallet) = &*WALLET.read() {
                                match wallet
                                    .read()
                                    .await
                                    .get_mnemonic(crate::wallet::utils::MnemonicLanguage::English)
                                    .await
                                {
                                    Ok(seed) => {
                                        info!("SeedPhrase: {seed}");
                                        seed_phrase_info.set(seed);
                                    }
                                    Err(e) => {
                                        info!("SeedPhrase: {e}");
                                        seed_phrase_info.set(e.to_string());
                                    }
                                }
                            } else {
                                info!("Wallet not initialized");
                                seed_phrase_info.set("Wallet not initialized".to_string());
                            }
                        } else {
                            info!("Incorrect password entered");
                            seed_phrase_info.set("Incorrect password entered".to_string());
                        }
                    }
                    Err(e) => {
                        info!("{e}");
                        seed_phrase_info.set(e.to_string());
                    }
                }
            }
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
