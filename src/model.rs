use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Deposit,
    Withdrawal,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub action: Action,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub id: u32,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct Account {
    #[serde(rename = "client")]
    pub id: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl Account {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }
}
