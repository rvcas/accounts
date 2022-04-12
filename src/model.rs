use std::ops::Neg;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub action: Action,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub id: u32,
    pub amount: Option<f64>,
    #[serde(default)]
    pub is_under_dispute: bool,
    #[serde(default)]
    pub failed: bool,
}

impl Transaction {
    pub fn is_withdrawal(&self) -> bool {
        matches!(self.action, Action::Withdrawal)
    }

    // I implemented this because I wasn't sure if disputes
    // could also happen on withdrawals. If not then no harm done
    // but if so then the amount needs to be negative.
    pub fn amount_with_sign(&self) -> Option<f64> {
        if self.is_withdrawal() {
            self.amount.map(f64::neg)
        } else {
            self.amount
        }
    }
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

    pub fn deposit(&mut self, amount: f64) {
        self.available += amount;
        self.total += amount;
    }

    pub fn withdraw(&mut self, amount: f64) {
        self.available -= amount;
        self.total -= amount;
    }

    pub fn dispute(&mut self, amount: f64) {
        self.available -= amount;
        self.held += amount;
    }

    pub fn resolve(&mut self, amount: f64) {
        self.available += amount;
        self.held -= amount;
    }

    pub fn chargeback(&mut self, amount: f64) {
        self.held -= amount;
        self.total -= amount;
        self.locked = true;
    }
}
