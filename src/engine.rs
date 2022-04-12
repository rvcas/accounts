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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit() {
        let mut engine = Engine::new();
        let entry = Transaction {
            action: Action::Deposit,
            client_id: 1,
            id: 1,
            amount: Some(10.0),
            is_under_dispute: false,
            failed: false,
        };

        engine.process(entry).unwrap();

        let account = engine.accounts.get(&1).unwrap();

        assert_eq!(account.available, 10.0);
        assert_eq!(account.total, 10.0);
        assert_eq!(account.held, 0.0);
        assert!(!account.locked);
    }

    #[test]
    fn withdraw() {
        let mut engine = Engine::new();
        let entries = vec![
            Transaction {
                action: Action::Deposit,
                client_id: 1,
                id: 1,
                amount: Some(10.0),
                is_under_dispute: false,
                failed: false,
            },
            Transaction {
                action: Action::Withdrawal,
                client_id: 1,
                id: 2,
                amount: Some(5.0),
                is_under_dispute: false,
                failed: false,
            },
        ];

        for entry in entries {
            engine.process(entry).unwrap();
        }

        let account = engine.accounts.get(&1).unwrap();

        assert_eq!(account.available, 5.0);
        assert_eq!(account.total, 5.0);
        assert_eq!(account.held, 0.0);
        assert!(!account.locked);
    }

    #[test]
    fn dispute() {
        let mut engine = Engine::new();
        let entries = vec![
            Transaction {
                action: Action::Deposit,
                client_id: 1,
                id: 1,
                amount: Some(10.0),
                is_under_dispute: false,
                failed: false,
            },
            Transaction {
                action: Action::Dispute,
                client_id: 1,
                id: 1,
                amount: None,
                is_under_dispute: false,
                failed: false,
            },
        ];

        for entry in entries {
            engine.process(entry).unwrap();
        }

        let account = engine.accounts.get(&1).unwrap();

        assert_eq!(account.available, 0.0);
        assert_eq!(account.held, 10.0);
        assert_eq!(account.total, 10.0);
        assert!(!account.locked);

        let transaction = engine.transactions.get(&1).unwrap();

        assert!(transaction.is_under_dispute);
    }

    #[test]
    fn resolve() {
        let mut engine = Engine::new();
        let entries = vec![
            Transaction {
                action: Action::Deposit,
                client_id: 1,
                id: 1,
                amount: Some(10.0),
                is_under_dispute: false,
                failed: false,
            },
            Transaction {
                action: Action::Dispute,
                client_id: 1,
                id: 1,
                amount: None,
                is_under_dispute: false,
                failed: false,
            },
            Transaction {
                action: Action::Resolve,
                client_id: 1,
                id: 1,
                amount: None,
                is_under_dispute: false,
                failed: false,
            },
        ];

        for entry in entries {
            engine.process(entry).unwrap();
        }

        let account = engine.accounts.get(&1).unwrap();

        assert_eq!(account.available, 10.0);
        assert_eq!(account.total, 10.0);
        assert_eq!(account.held, 0.0);
        assert!(!account.locked);

        let transaction = engine.transactions.get(&1).unwrap();

        assert!(!transaction.is_under_dispute);
    }

    #[test]
    fn chargeback() {
        let mut engine = Engine::new();
        let entries = vec![
            Transaction {
                action: Action::Deposit,
                client_id: 1,
                id: 1,
                amount: Some(10.0),
                is_under_dispute: false,
                failed: false,
            },
            Transaction {
                action: Action::Dispute,
                client_id: 1,
                id: 1,
                amount: None,
                is_under_dispute: false,
                failed: false,
            },
            Transaction {
                action: Action::Chargeback,
                client_id: 1,
                id: 1,
                amount: None,
                is_under_dispute: false,
                failed: false,
            },
        ];

        for entry in entries {
            engine.process(entry).unwrap();
        }

        let account = engine.accounts.get(&1).unwrap();

        assert_eq!(account.available, 0.0);
        assert_eq!(account.held, 0.0);
        assert_eq!(account.total, 0.0);
        assert!(account.locked);

        let transaction = engine.transactions.get(&1).unwrap();

        assert!(!transaction.is_under_dispute);
    }

    #[test]
    fn withdraw_more_than_available() {
        let mut engine = Engine::new();
        let entries = vec![
            Transaction {
                action: Action::Deposit,
                client_id: 1,
                id: 1,
                amount: Some(10.0),
                is_under_dispute: false,
                failed: false,
            },
            Transaction {
                action: Action::Withdrawal,
                client_id: 1,
                id: 2,
                amount: Some(15.0),
                is_under_dispute: false,
                failed: false,
            },
        ];

        for entry in entries {
            engine.process(entry).unwrap();
        }

        let account = engine.accounts.get(&1).unwrap();

        assert_eq!(account.available, 10.0);
        assert_eq!(account.total, 10.0);
        assert_eq!(account.held, 0.0);
        assert!(!account.locked);

        let transaction = engine.transactions.get(&2).unwrap();

        assert!(transaction.failed);
    }
}
