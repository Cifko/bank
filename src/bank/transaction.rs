//! Transaction module for handling various types of banking transactions.
use serde::{Deserialize, de};

use crate::bank::{
    DECIMAL_PRECISION, TransactionId,
    types::{ClientId, Money},
};

/// Enum representing the type of transaction.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Custom deserializer for monetary values to handle fixed-point representation.
fn deserialize_money<'de, D>(deserializer: D) -> Result<Option<Money>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value: Option<f64> = Option::deserialize(deserializer)?;
    Ok(value.map(|v| (v * DECIMAL_PRECISION) as Money))
}

/// Represents a banking transaction.
#[derive(Deserialize, Debug, Clone)]
pub struct Transaction {
    /// The type of transaction (e.g., Deposit, Withdrawal, etc.)
    #[serde(rename = "type")]
    tx_type: TransactionType,

    /// The unique identifier for the client associated with this transaction.
    #[serde(rename = "client")]
    client_id: ClientId,

    /// The unique identifier for this transaction.
    #[serde(rename = "tx")]
    transaction_id: TransactionId,

    /// The amount involved in the transaction, if applicable.
    #[serde(rename = "amount", deserialize_with = "deserialize_money")]
    amount: Option<Money>,
}

impl Transaction {
    /// Gets the type of the transaction.
    pub fn get_type(&self) -> &TransactionType {
        &self.tx_type
    }

    /// Gets the amount of the transaction, if applicable.
    pub fn get_amount(&self) -> Option<Money> {
        self.amount
    }

    /// Gets the transaction ID.
    pub fn get_transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    /// Gets the client ID associated with this transaction.
    pub fn get_client_id(&self) -> ClientId {
        self.client_id
    }

    #[cfg(test)]
    pub fn new(
        tx_type: TransactionType,
        client_id: ClientId,
        transaction_id: TransactionId,
        amount: Option<Money>,
    ) -> Self {
        Transaction {
            tx_type,
            client_id,
            transaction_id,
            amount,
        }
    }
}
