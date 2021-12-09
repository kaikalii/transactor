use std::{error::Error, fmt, str::FromStr};

use crate::amount::Amount;

pub type ClientId = u16;
pub type TransactionId = u32;

/// A transaction to be executed on `Accounts`
#[derive(Debug)]
pub struct Transaction {
    pub client: ClientId,
    pub id: TransactionId,
    pub ty: TransactionType,
}

/// A type of transaction
#[derive(Debug, Clone)]
pub enum TransactionType {
    /// A deposit into an account
    Deposit(Amount),
    /// A withdrawal from an account
    Withdrawal(Amount),
    /// A dispute on some transaction. Disputed funds go into holding.
    ///
    /// Currently, only deposits can be disputed
    Dispute,
    /// Resolve a dispute. Funds held by the dispute become available again.
    ///
    /// Does nothing if the referenced transaction id does not exist or it is not a deposit
    Resolve,
    /// Charge back a disputed amount. Funds held by the dispute are removed.
    ///
    /// Does nothing if the referenced transaction id does not exist or it is not a deposit
    Chargeback,
}

#[derive(Debug)]
pub enum TransactionParseError {
    MissingTransactionType,
    InvalidTransactionType(String),
    MissingClientId,
    InvalidClientId(String),
    MissingTransactionId,
    InvalidTransactionId(String),
    MissingAmount,
    InvalidAmount(String),
}

impl fmt::Display for TransactionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionParseError::MissingTransactionType => write!(f, "Missing transaction type"),
            TransactionParseError::InvalidTransactionType(s) => {
                write!(f, "Invalid transaction type {:?}", s)
            }
            TransactionParseError::MissingClientId => write!(f, "Missing client id"),
            TransactionParseError::InvalidClientId(s) => write!(f, "Invalid client id {:?}", s),
            TransactionParseError::MissingTransactionId => write!(f, "Missing transaction id"),
            TransactionParseError::InvalidTransactionId(s) => {
                write!(f, "Invalid transaction id {:?}", s)
            }
            TransactionParseError::MissingAmount => write!(f, "Missing amount"),
            TransactionParseError::InvalidAmount(s) => write!(f, "Invalid amount {:?}", s),
        }
    }
}

impl Error for TransactionParseError {}

impl FromStr for Transaction {
    type Err = TransactionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(',').map(str::trim);
        let transaction_type = parts
            .next()
            .ok_or(TransactionParseError::MissingTransactionType)?;
        let client_id = parts.next().ok_or(TransactionParseError::MissingClientId)?;
        let client_id = client_id
            .parse::<ClientId>()
            .map_err(|_| TransactionParseError::InvalidClientId(client_id.into()))?;
        let transaction_id = parts
            .next()
            .ok_or(TransactionParseError::MissingTransactionId)?;
        let transaction_id = transaction_id
            .parse::<TransactionId>()
            .map_err(|_| TransactionParseError::InvalidTransactionId(transaction_id.into()))?;
        let mut amount = || -> Result<Amount, Self::Err> {
            let amount_str = parts.next().ok_or(TransactionParseError::MissingAmount)?;
            let amount = amount_str
                .parse::<f64>()
                .map_err(|_| TransactionParseError::InvalidTransactionId(amount_str.into()))?;
            Amount::from_f64(amount)
                .ok_or_else(|| TransactionParseError::InvalidAmount(amount_str.into()))
        };
        let transaction_type = match transaction_type {
            "deposit" => TransactionType::Deposit(amount()?),
            "withdrawal" => TransactionType::Withdrawal(amount()?),
            "dispute" => TransactionType::Dispute,
            "resolve" => TransactionType::Resolve,
            "chargeback" => TransactionType::Chargeback,
            _ => {
                return Err(TransactionParseError::InvalidTransactionType(
                    transaction_type.into(),
                ))
            }
        };
        Ok(Transaction {
            client: client_id,
            id: transaction_id,
            ty: transaction_type,
        })
    }
}
