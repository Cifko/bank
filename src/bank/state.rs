//! The `State` module manages the accounts and processes transactions in a banking system.
use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::bank::{Account, ClientId, Transaction, TransactionError};

/// Represents the state of the banking system, including all accounts.
pub struct State {
    /// A map of client IDs to their respective accounts.
    accounts: HashMap<ClientId, Account>,
    /// A channel receiver for processing incoming transactions.
    receiver: mpsc::Receiver<Transaction>,
}

impl State {
    /// Creates a new instance of `State` with an empty accounts map.
    pub fn new(receiver: mpsc::Receiver<Transaction>) -> Self {
        State {
            accounts: HashMap::new(),
            receiver,
        }
    }

    /// Retrieves an account by client ID, or creates a new one if it doesn't exist.
    pub fn get_or_create_account(&mut self, client_id: ClientId) -> &mut Account {
        self.accounts
            .entry(client_id)
            .or_insert(Account::new(client_id))
    }

    /// Retrieves all accounts in the state.
    pub fn get_all_accounts(&self) -> &HashMap<ClientId, Account> {
        &self.accounts
    }

    /// Processes a transaction, updating the account state accordingly.
    fn process_transaction(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        let account = self.get_or_create_account(transaction.get_client_id());
        account.process_transaction(transaction)
    }

    /// Runs the state management loop, processing transactions from the receiver.
    pub async fn run(&mut self) {
        while let Some(transaction) = self.receiver.recv().await {
            if let Err(e) = self.process_transaction(transaction) {
                eprintln!("Error processing transaction: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bank::{Transaction, TransactionType};

    #[tokio::test]
    async fn test_account_creation() {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        let mut state = super::State::new(receiver);
        assert!(state.get_all_accounts().is_empty());
        sender
            .send(Transaction::new(TransactionType::Deposit, 1, 1, Some(1000)))
            .await
            .unwrap();
        drop(sender); // Close the sender to signal no more transactions will be sent
        state.run().await;
        let accounts = state.get_all_accounts();
        assert_eq!(accounts.len(), 1);
        assert!(accounts.contains_key(&1));
    }
}
