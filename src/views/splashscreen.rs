use crate::database::sql::DbOpenWallet;
use dioxus::prelude::*;

#[component]
pub fn SplashScreen() -> Element {
    rsx!(
        div {"Splash Screen"}
        div {DbOpenWallet {}}
    )
}
