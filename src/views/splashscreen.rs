use crate::database::sql::db_open_wallet;
use dioxus::prelude::*;

#[component]
pub fn SplashScreen() -> Element {
    // open the wallet
    use_future(move || async move {
        db_open_wallet().await;
    });

    rsx!(
        div {"Splash Screen"}
    )
}
