use std::io;

use accounts::{cli::Cli, engine::Engine, model::Transaction};

fn main() -> anyhow::Result<()> {
    let args = Cli::default();

    let mut engine = Engine::new();

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(args.input)?;

    for result in rdr.deserialize() {
        let entry: Transaction = result?;

        engine.process(entry)?;
    }

    let mut wtr = csv::Writer::from_writer(io::stdout());

    for account in engine.accounts.values() {
        wtr.serialize(account)?;
    }

    wtr.flush()?;

    Ok(())
}
