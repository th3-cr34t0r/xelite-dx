use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::{FromRow, Type},
    Decode,
};

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
    pub name: String,
    pub address: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DbRemoveContact {
    address: String,
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

#[derive(Default, Serialize, Deserialize, Debug, Clone, FromRow, Type)]
pub struct DbMessage {
    pub status: String,
    pub direction: String,
    pub address: String,
    pub hash: String,
    pub fee: f64,
    pub timestamp: i64,
    pub topoheight: i64,
    pub asset: String,
    pub amount: i64,
    pub message: Option<String>,
}
