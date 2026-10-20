use crate::database::db_fns::db_open_wallet;
use dioxus::prelude::*;

#[component]
pub fn SplashScreen() -> Element {
    use_future(move || async move {
        db_open_wallet().await;
    });

    rsx!(
        div {
            style: "display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:100vh; background-color:#0a0a0a; color:#ffffff;",
            h1 { style: "font-size:48px; font-weight:bold; margin-bottom:16px;", "Xelite DX" }
            p { style: "font-size:20px; color:#9ca3af;", "Connecting..." }
        }
    )
}
