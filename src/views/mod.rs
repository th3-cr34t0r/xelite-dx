use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

pub mod chat_view;
pub mod home;
pub mod restore_wallet_options;
pub mod splashscreen;

#[derive(Serialize, Deserialize)]
pub struct WalletCreateOpenArgs<'a> {
    name: &'a str,
    password: &'a str,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct DbContact {
    name: String,
    address: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DbRemoveContact {
    address: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum MessageDirection {
    #[default]
    Err,
    Incoming,
    Outgoing,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DbMessage {
    address: String,
    direction: MessageDirection,
    topoheight: i64,
    timestamp: i64,
    message: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DbAddress {
    address: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DbPassword {
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TxSendMsg {
    message: String,
    address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TxData {
    topoheight: u64,
    hash: String,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppEvents {
    topoheight: u64,
    transactions: Vec<TxData>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct MsgTransfer {
    asset: String,
    amount: u64,
    message: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct RxMsg {
    from_address: String,
    hash: String,
    timestamp: u64,
    topoheight: u64,
    transfer: MsgTransfer,
}
