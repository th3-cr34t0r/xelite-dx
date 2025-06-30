use crate::database::sql::db_open_wallet;
use dioxus::prelude::*;

#[component]
pub fn SplashScreen() -> Element {
    // open the wallet
    db_open_wallet();

    rsx!(
        div {"Splash Screen"}
    )
}
