use std::{collections::HashMap, io};

use accounts::{
    cli::Cli,
    model::{Account, Action, Transaction},
};

fn main() -> anyhow::Result<()> {
    let args = Cli::default();

    let raw_entries = std::fs::read(&args.input)?;

    let mut transactions = HashMap::new();
    let mut accounts = HashMap::new();

    let mut rdr = csv::Reader::from_reader(raw_entries.as_slice());

    for result in rdr.deserialize() {
        let entry: Transaction = result?;

        let transaction = transactions.entry(entry.id).or_insert(entry);

        let account = accounts
            .entry(entry.client_id)
            .or_insert_with(|| Account::new(entry.client_id));

        match entry.action {
            Action::Deposit => {
                account.available += entry.amount;
                account.total += entry.amount;
            }
            Action::Withdraw => {
                account.available -= entry.amount;
                account.total -= entry.amount;
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
