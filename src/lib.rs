use std::fs::File;

use csv::{ReaderBuilder, Trim, Writer};

mod account;
pub use account::AccountOutput;

mod engine;
pub use engine::Engine;

mod error;
pub use error::EngineError;

mod transaction;
pub use transaction::{InputRecord, Transaction};

pub type ClientId = u16;
pub type TransactionId = u32;

/// Processes a CSV file containing transactions and writes the account state to a writer
pub fn process_csv<P, W>(path: P, writer: W) -> Result<(), EngineError>
where
    P: AsRef<std::path::Path>,
    W: std::io::Write,
{
    let file = File::open(path)?;
    process_reader(file, writer)
}

fn process_reader<R, W>(reader: R, writer: W) -> Result<(), EngineError>
where
    R: std::io::Read,
    W: std::io::Write,
{
    let mut csv_reader = ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_reader(reader);

    let mut engine = Engine::new();

    for result in csv_reader.deserialize() {
        let input: InputRecord = result?;
        engine.process_tx(Transaction::try_from(input)?);
    }

    let mut csv_writer = Writer::from_writer(writer);

    for account in engine.accounts() {
        csv_writer.serialize(&account)?;
    }

    csv_writer.flush()?;

    Ok(())
}
