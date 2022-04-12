use std::collections::HashMap;

use crate::model::{Account, Action, Transaction};

pub struct Engine {
    pub transactions: HashMap<u32, Transaction>,
    pub accounts: HashMap<u16, Account>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            accounts: HashMap::new(),
        }
    }

    pub fn process(&mut self, mut entry: Transaction) -> anyhow::Result<()> {
        let account = self
            .accounts
            .entry(entry.client_id)
            .or_insert_with(|| Account::new(entry.client_id));

        match entry.action {
            Action::Deposit => {
                if let Some(amount) = entry.amount {
                    account.deposit(amount);

                    self.transactions.insert(entry.id, entry);
                } else {
                    return Err(anyhow::anyhow!("Deposit transaction without amount"));
                }
            }
            Action::Withdrawal => {
                if let Some(amount) = entry.amount {
                    if amount <= account.available {
                        account.withdraw(amount);
                    } else {
                        entry.failed = true;
                    }

                    self.transactions.insert(entry.id, entry);
                } else {
                    return Err(anyhow::anyhow!("Withdraw transaction without amount"));
                }
            }
            Action::Dispute => {
                if let Some(transaction) = self.transactions.get_mut(&entry.id) {
                    if transaction.client_id == account.id {
                        transaction.is_under_dispute = true;

                        if let Some(amount) = transaction.amount_with_sign() {
                            account.dispute(amount);
                        } else {
                            return Err(anyhow::anyhow!(
                                "Dispute references transaction without amount"
                            ));
                        }
                    }
                }
            }
            Action::Resolve => {
                if let Some(transaction) = self.transactions.get_mut(&entry.id) {
                    if transaction.is_under_dispute && transaction.client_id == account.id {
                        transaction.is_under_dispute = false;

                        if let Some(amount) = transaction.amount_with_sign() {
                            account.resolve(amount);
                        } else {
                            return Err(anyhow::anyhow!(
                                "Resolve references transaction without amount"
                            ));
                        }
                    }
                }
            }
            Action::Chargeback => {
                if let Some(transaction) = self.transactions.get_mut(&entry.id) {
                    if transaction.is_under_dispute && transaction.client_id == account.id {
                        transaction.is_under_dispute = false;

                        if let Some(amount) = transaction.amount_with_sign() {
                            account.chargeback(amount);
                        } else {
                            return Err(anyhow::anyhow!(
                                "Dispute references transaction without amount"
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
