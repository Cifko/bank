use csv::{ReaderBuilder, Trim};
use tokio::sync::mpsc;

mod bank;

/// The size of the channel for processing transactions.
const CHANNEL_SIZE: usize = 100;

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_csv_file>", args[0]);
        std::process::exit(1);
    }
    let input_file = &args[1];

    let (sender, receiver) = mpsc::channel(CHANNEL_SIZE);
    let mut state = bank::State::new(receiver);

    let handle = tokio::spawn(async move {
        state.run().await;
        state
    });

    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(input_file)
        .expect("Failed to read CSV file");

    for transaction in reader.deserialize().flatten() {
        if let Err(err) = sender.send(transaction).await {
            eprintln!("Error sending transaction: {err}");
        }
    }

    drop(sender); // Close the sender to signal no more transactions will be sent
    let state = handle
        .await
        .expect("Failed to join the state handling task");

    let mut writer = csv::Writer::from_writer(std::io::stdout());
    for account in state.get_all_accounts().values() {
        if let Err(err) = writer.serialize(account) {
            eprintln!("Error writing account: {err}");
        }
    }
}
