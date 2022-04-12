use std::{collections::HashMap, io};

use accounts::{
    cli::Cli,
    model::{Account, Action, Transaction},
};

fn main() -> anyhow::Result<()> {
    let args = Cli::default();

    let mut transactions = HashMap::new();
    let mut accounts = HashMap::new();

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(args.input)?;

    for result in rdr.deserialize() {
        let mut entry: Transaction = result?;

        let account = accounts
            .entry(entry.client_id)
            .or_insert_with(|| Account::new(entry.client_id));

        match entry.action {
            Action::Deposit => {
                if let Some(amount) = entry.amount {
                    account.deposit(amount);

                    transactions.insert(entry.id, entry);
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

                    transactions.insert(entry.id, entry);
                } else {
                    return Err(anyhow::anyhow!("Withdraw transaction without amount"));
                }
            }
            Action::Dispute => {
                if let Some(transaction) = transactions.get_mut(&entry.id) {
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
                if let Some(transaction) = transactions.get_mut(&entry.id) {
                    if transaction.is_under_dispute && transaction.client_id == account.id {
                        transaction.is_under_dispute = false;

                        if let Some(amount) = transaction.amount_with_sign() {
                            account.resolve(amount);
                        } else {
                            return Err(anyhow::anyhow!(
                                "Dispute references transaction without amount"
                            ));
                        }
                    }
                }
            }
            Action::Chargeback => {
                if let Some(transaction) = transactions.get_mut(&entry.id) {
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
    }

    let mut wtr = csv::Writer::from_writer(io::stdout());

    for account in accounts.values() {
        wtr.serialize(account)?;
    }

    wtr.flush()?;

    Ok(())
}
