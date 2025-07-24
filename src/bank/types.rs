//! Types used throughout the banking system.

/// Decimal precision for monetary values.
/// This is used to convert floating-point values to fixed-point representation.
pub const DECIMAL_PRECISION: f64 = 10000.0;

/// Client ID type, representing a unique identifier for a client.
pub type ClientId = u16;

/// Transaction ID type, representing a unique identifier for a transaction.
pub type TransactionId = u32;

/// Money type, representing a fixed-point monetary value.
pub type Money = i64;
