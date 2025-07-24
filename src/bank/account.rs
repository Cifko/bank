//! Account management and transaction processing for a banking system.
use std::collections::{HashMap, HashSet};

use serde::Serialize;
use thiserror::Error;

use crate::bank::{
    DECIMAL_PRECISION, Transaction, TransactionId, TransactionType,
    types::{ClientId, Money},
};

fn serialize_money<S>(money: &Money, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    (*money as f64 / DECIMAL_PRECISION).serialize(serializer)
}

/// Represents a bank account for a client.
#[derive(Serialize, Default)]
pub struct Account {
    /// The unique identifier for the client.
    #[serde(rename = "client")]
    client_id: ClientId,

    /// The available balance in the account.
    #[serde(serialize_with = "serialize_money")]
    available: Money,

    /// The held amount in the account for disputed transactions.
    #[serde(serialize_with = "serialize_money")]
    held: Money,

    /// The total balance in the account, including available and held amounts.
    #[serde(serialize_with = "serialize_money")]
    total: Money,

    /// Indicates whether the account is locked.
    locked: bool,

    /// A map of transactions associated with this account.
    #[serde(skip)]
    transactions: HashMap<TransactionId, Transaction>,

    /// A set of transaction IDs that are currently in dispute.
    #[serde(skip)]
    in_dispute: HashSet<TransactionId>,
}

impl Account {
    /// Creates a new account for the given client ID.
    pub fn new(client_id: ClientId) -> Self {
        Account {
            client_id,
            ..Default::default()
        }
    }

    /// Deposits the specified amount into the account.
    fn deposit(&mut self, amount: Money) {
        self.available += amount;
        self.total += amount;
    }

    /// Withdraws the specified amount from the account. Returns an error if there are insufficient funds.
    fn withdraw(&mut self, amount: Money) -> Result<(), TransactionError> {
        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;
            Ok(())
        } else {
            Err(TransactionError::InsufficientFunds)
        }
    }

    /// Marks a transaction as disputed. If the transaction is a deposit, it moves the amount from available to held. If it's a withdrawal, it adds the amount to held.
    /// Returns an error if the transaction is already in dispute or if the transaction doesn't exists.
    fn dispute(&mut self, transaction_id: TransactionId) -> Result<(), TransactionError> {
        if self.in_dispute.contains(&transaction_id) {
            return Err(TransactionError::AlreadyInDispute);
        }
        if let Some(tx) = self.transactions.get(&transaction_id) {
            match tx.get_type() {
                TransactionType::Deposit => {
                    self.available -= tx.get_amount().unwrap_or(0);
                    self.held += tx.get_amount().unwrap_or(0);
                }
                TransactionType::Withdrawal => {
                    self.held += tx.get_amount().unwrap_or(0);
                }
                _ => return Err(TransactionError::InvalidTransaction),
            }
            self.in_dispute.insert(transaction_id);
            Ok(())
        } else {
            Err(TransactionError::TransactionDoesNotExist)
        }
    }

    /// Resolves a disputed transaction, moving the amount back to available if it was a deposit, or reducing held if it was a withdrawal.
    /// Returns an error if the transaction is not in dispute or if the transaction doesn't exist.
    fn resolve(&mut self, transaction_id: TransactionId) -> Result<(), TransactionError> {
        if !self.in_dispute.contains(&transaction_id) {
            return Err(TransactionError::NotInDispute);
        }
        if let Some(tx) = self.transactions.get(&transaction_id) {
            match tx.get_type() {
                TransactionType::Deposit => {
                    self.available += tx.get_amount().unwrap_or(0);
                    self.held -= tx.get_amount().unwrap_or(0);
                }
                TransactionType::Withdrawal => {
                    self.held -= tx.get_amount().unwrap_or(0);
                }
                _ => return Err(TransactionError::InvalidTransaction),
            }
            self.in_dispute.remove(&transaction_id);
            Ok(())
        } else {
            Err(TransactionError::TransactionDoesNotExist)
        }
    }

    /// Charges back a disputed transaction, locking the account and moving the held amount to total if it was a deposit, or returning the held amount to available if it was a withdrawal.
    /// Returns an error if the transaction is not in dispute or if the transaction doesn't exist.
    fn chargeback(&mut self, transaction_id: TransactionId) -> Result<(), TransactionError> {
        if !self.in_dispute.contains(&transaction_id) {
            return Err(TransactionError::NotInDispute);
        }
        if let Some(tx) = self.transactions.get(&transaction_id) {
            match tx.get_type() {
                TransactionType::Deposit => {
                    self.held -= tx.get_amount().unwrap_or_default();
                    self.total -= tx.get_amount().unwrap_or_default();
                }
                TransactionType::Withdrawal => {
                    self.available += tx.get_amount().unwrap_or_default();
                    self.held -= tx.get_amount().unwrap_or_default();
                }
                _ => return Err(TransactionError::InvalidTransaction),
            }
            self.locked = true;
            self.in_dispute.remove(&transaction_id);
            Ok(())
        } else {
            Err(TransactionError::TransactionDoesNotExist)
        }
    }

    /// Processes a transaction based on its type.
    /// Returns an error if the account is locked or if the transaction is invalid.
    pub fn process_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), TransactionError> {
        if transaction.get_client_id() != self.client_id {
            return Err(TransactionError::NotForThisAccount);
        }

        if self.locked {
            return Err(TransactionError::AccountLocked);
        }

        match transaction.get_type() {
            TransactionType::Deposit => {
                let amount = transaction
                    .get_amount()
                    .ok_or(TransactionError::InvalidTransaction)?;
                self.deposit(amount);
                self.transactions
                    .insert(transaction.get_transaction_id(), transaction);
            }
            TransactionType::Withdrawal => {
                let amount = transaction
                    .get_amount()
                    .ok_or(TransactionError::InvalidTransaction)?;
                self.withdraw(amount)?;
                self.transactions
                    .insert(transaction.get_transaction_id(), transaction);
            }
            TransactionType::Dispute => {
                self.dispute(transaction.get_transaction_id())?;
            }
            TransactionType::Resolve => {
                self.resolve(transaction.get_transaction_id())?;
            }
            TransactionType::Chargeback => {
                self.chargeback(transaction.get_transaction_id())?;
            }
        }
        Ok(())
    }
}

/// Errors that can occur during transaction processing.
#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Insufficient funds for transaction")]
    InsufficientFunds,
    #[error("Account is locked")]
    AccountLocked,
    #[error("Invalid transaction")]
    InvalidTransaction,
    #[error("Transaction is already in dispute")]
    AlreadyInDispute,
    #[error("Transaction not in dispute")]
    NotInDispute,
    #[error("Transaction is not for this account")]
    NotForThisAccount,
    #[error("Transaction does not exist")]
    TransactionDoesNotExist,
}

#[cfg(test)]
mod tests {
    use crate::bank::{Account, TransactionError, TransactionType, transaction::Transaction};

    #[test]
    fn test_wrong_account() {
        let mut account = Account::new(1);
        let transaction = Transaction::new(
            TransactionType::Deposit,
            2, // Different client ID
            1,
            Some(1000),
        );
        assert!(matches!(
            account.process_transaction(transaction),
            Err(TransactionError::NotForThisAccount)
        ));
    }

    #[test]
    fn test_deposit() {
        let mut account = Account::new(1);
        let transaction = Transaction::new(TransactionType::Deposit, 1, 2, Some(1000));
        assert!(account.process_transaction(transaction).is_ok());
        assert_eq!(account.available, 1000);
        assert_eq!(account.total, 1000);
    }

    #[test]
    fn test_withdrawal() {
        let mut account = Account::new(1);
        account.deposit(2000);
        let transaction = Transaction::new(TransactionType::Withdrawal, 1, 2, Some(1000));
        assert!(account.process_transaction(transaction).is_ok());
        assert_eq!(account.available, 1000);
        assert_eq!(account.total, 1000);
    }

    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut account = Account::new(1);
        let transaction = Transaction::new(TransactionType::Withdrawal, 1, 2, Some(1000));
        assert!(matches!(
            account.process_transaction(transaction),
            Err(TransactionError::InsufficientFunds)
        ));
    }

    #[test]
    fn test_invalid_dispute() {
        let mut account = Account::new(1);
        let transaction = Transaction::new(TransactionType::Dispute, 1, 2, None);
        assert!(matches!(
            account.process_transaction(transaction),
            Err(TransactionError::TransactionDoesNotExist)
        ));
    }

    #[test]
    fn test_dispute() {
        let mut account = Account::new(1);
        let transaction = Transaction::new(TransactionType::Deposit, 1, 2, Some(1000));
        assert!(account.process_transaction(transaction).is_ok());
        let dispute_tx = Transaction::new(TransactionType::Dispute, 1, 2, None);
        assert!(account.process_transaction(dispute_tx).is_ok());
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 1000);
    }

    #[test]
    fn test_double_dispute() {
        let mut account = Account::new(1);
        let transaction = Transaction::new(TransactionType::Deposit, 1, 2, Some(1000));
        assert!(account.process_transaction(transaction).is_ok());
        let dispute_tx = Transaction::new(TransactionType::Dispute, 1, 2, None);
        assert!(account.process_transaction(dispute_tx.clone()).is_ok());
        assert!(matches!(
            account.process_transaction(dispute_tx),
            Err(TransactionError::AlreadyInDispute)
        ));
    }

    #[test]
    fn test_resolve() {
        let mut account = Account::new(1);
        let transaction = Transaction::new(TransactionType::Deposit, 1, 2, Some(1000));
        assert!(account.process_transaction(transaction).is_ok());
        let dispute_tx = Transaction::new(TransactionType::Dispute, 1, 2, None);
        assert!(account.process_transaction(dispute_tx).is_ok());
        let resolve_tx = Transaction::new(TransactionType::Resolve, 1, 2, None);
        assert!(account.process_transaction(resolve_tx).is_ok());
        assert_eq!(account.available, 1000);
        assert_eq!(account.held, 0);
    }

    #[test]
    fn test_deposit_chargeback() {
        let mut account = Account::new(1);
        let transaction = Transaction::new(TransactionType::Deposit, 1, 2, Some(1000));
        assert!(account.process_transaction(transaction).is_ok());
        let dispute_tx = Transaction::new(TransactionType::Dispute, 1, 2, None);
        assert!(account.process_transaction(dispute_tx).is_ok());
        let chargeback_tx = Transaction::new(TransactionType::Chargeback, 1, 2, None);
        assert!(account.process_transaction(chargeback_tx).is_ok());
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 0);
        assert!(account.locked);
    }

    #[test]
    fn test_withdraw_chargeback() {
        let mut account = Account::new(1);
        account.deposit(2000);
        let transaction = Transaction::new(TransactionType::Withdrawal, 1, 2, Some(1000));
        assert!(account.process_transaction(transaction).is_ok());
        let dispute_tx = Transaction::new(TransactionType::Dispute, 1, 2, None);
        assert!(account.process_transaction(dispute_tx).is_ok());
        let chargeback_tx = Transaction::new(TransactionType::Chargeback, 1, 2, None);
        assert!(account.process_transaction(chargeback_tx).is_ok());
        assert_eq!(account.available, 2000);
        assert_eq!(account.held, 0);
        assert!(account.locked);
    }
}
