// The dioxus prelude contains a ton of common items used in dioxus apps. It's a good idea to import wherever you
// need dioxus
use dioxus::prelude::*;

use sqlx::SqlitePool;
use tokio::sync::RwLock;
use views::{
    chat_view::ChatView,
    home::{AddContact, Home, ViewSeed},
    restore_wallet_options::{
        CreateNewWallet, RestoreFromPrivateKey, RestoreFromSeed, RestoreWalletOptions,
    },
    splashscreen::SplashScreen,
};

use crate::wallet::utils::ChatWallet;
/// Define a components module that contains all shared components for our app.
mod components;
mod database;
/// Define a views module that contains the UI for all Layouts and Routes for our app.
mod views;
mod wallet;
/// The Route enum is used to define the structure of internal routes in our app. All route enums need to derive
/// the [`Routable`] trait, which provides the necessary methods for the router to work.
/// 
/// Each variant represents a different URL pattern that can be matched by the router. If that pattern is matched,
/// the components for that route will be rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    // The layout attribute defines a wrapper for all routes under the layout. Layouts are great for wrapping
    // many routes with a common UI like a navbar.
    #[route("/")]
    SplashScreen {},
    #[route("/restorewalletoptions")]
    RestoreWalletOptions {},
    #[route("/createnewwallet")]
    CreateNewWallet {},
    #[route("/restorefromseed")]
    RestoreFromSeed {},
    #[route("/restorefromprivkey")]
    RestoreFromPrivateKey {},
    #[route("/home")]
    Home {},
    #[route("/chatview?:name&:address")]
    ChatView { name: String, address: String },
    #[route("/addcontact")]
    AddContact {},
    #[route("/viewseed")]
    ViewSeed {},
}

// We can import assets in dioxus with the `asset!` macro. This macro takes a path to an asset relative to the crate root.
// The macro returns an `Asset` type that will display as the path to the asset in the browser or a local path in desktop bundles.
const FAVICON: Asset = asset!("/assets/favicon.ico");
// The asset macro also minifies some assets like CSS and JS to make bundled smaller
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

pub static DB: GlobalSignal<Option<SqlitePool>> = Signal::global(|| None);
pub static WALLET: GlobalSignal<Option<RwLock<ChatWallet>>> = Signal::global(|| None);
pub static IS_READY: GlobalSignal<RwLock<bool>> = Signal::global(|| RwLock::new(true));

fn main() {
    // call to fix crypto provider issue
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // The `launch` function is the main entry point for a dioxus app. It takes a component and renders it with the platform feature
    // you have enabled
    dioxus::launch(App);
}

/// App is the main component of our app. Components are the building blocks of dioxus apps. Each component is a function
/// that takes some props and returns an Element. In this case, App takes no props because it is the root of our app.
///
/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
#[component]
fn App() -> Element {
    // The `rsx!` macro lets us define HTML inside of rust. It expands to an Element with all of our HTML inside.
    rsx!(

        document::Stylesheet{
            href: asset!("/assets/tailwind.css")
        }

        div { class:"app-container flex flex-col overflow-hidden",

            Router::<Route> {}
        }
    )
}
