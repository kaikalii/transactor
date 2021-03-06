//! Transaction types

use std::{error::Error, fmt, str::FromStr};

use crate::amount::Amount;

pub type ClientId = u16;
pub type TransactionId = u32;

/// A client-specific transaction to be executed on [`Accounts`]
#[derive(Debug, Clone)]
pub struct ClientTransaction {
    pub client: ClientId,
    pub tx: Transaction,
}

/// A transaction type for a standard deposit or withdrawal
#[derive(Debug, Clone, Copy)]
pub enum ChangeKind {
    Deposit,
    Withdrawal,
}

/// A change to a balance
#[derive(Debug, Clone, Copy)]
pub struct BalanceChange {
    pub kind: ChangeKind,
    pub amount: Amount,
}

/// A transaction type for resolving disputes
#[derive(Debug, Clone, Copy)]
pub enum ResolutionKind {
    /// Resolve a dispute by reverting it. Funds held by the dispute become available again.
    ///
    /// Does nothing if the referenced transaction id does not exist or is not a deposit
    Resolve,
    /// Charge back a disputed amount. Funds held by the dispute are removed.
    ///
    /// Does nothing if the referenced transaction id does not exist or is not a deposit
    Chargeback,
}

/// A transaction to be executed on an [`Account`]
#[derive(Debug, Clone, Copy)]
pub enum Transaction {
    /// A deposit or withdrawal into an account
    Change {
        tx_id: TransactionId,
        change: BalanceChange,
    },
    /// Initiate a dispute on some transaction. Disputed funds go into holding.
    ///
    /// Currently, only deposits can be disputed
    Dispute(TransactionId),
    /// A resolution to a dispute
    Resolution {
        kind: ResolutionKind,
        tx_id: TransactionId,
    },
}

impl Transaction {
    pub const fn change(tx_id: TransactionId, kind: ChangeKind, amount: Amount) -> Transaction {
        Transaction::Change {
            tx_id,
            change: BalanceChange { kind, amount },
        }
    }
    pub const fn deposit(tx_id: TransactionId, amount: Amount) -> Transaction {
        Transaction::change(tx_id, ChangeKind::Deposit, amount)
    }
    pub const fn withdrawal(tx_id: TransactionId, amount: Amount) -> Transaction {
        Transaction::change(tx_id, ChangeKind::Withdrawal, amount)
    }
    pub const fn resolution(kind: ResolutionKind, tx_id: TransactionId) -> Transaction {
        Transaction::Resolution { kind, tx_id }
    }
}

/// An error that can occur when attempting to parse a `ClientTransaction` from a comma-separated string
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

impl FromStr for ClientTransaction {
    type Err = TransactionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(',').map(str::trim);
        // Get the transaction type string
        let tx_type = parts
            .next()
            .ok_or(TransactionParseError::MissingTransactionType)?;
        // Parse client id
        let client_id = parts.next().ok_or(TransactionParseError::MissingClientId)?;
        let client_id = client_id
            .parse::<ClientId>()
            .map_err(|_| TransactionParseError::InvalidClientId(client_id.into()))?;
        // Parse transaction id
        let tx_id = parts
            .next()
            .ok_or(TransactionParseError::MissingTransactionId)?;
        let tx_id = tx_id
            .parse::<TransactionId>()
            .map_err(|_| TransactionParseError::InvalidTransactionId(tx_id.into()))?;
        // Closure for getting the amount
        let mut amount = || -> Result<Amount, Self::Err> {
            let amount_str = parts.next().ok_or(TransactionParseError::MissingAmount)?;
            let amount = amount_str
                .parse::<f64>()
                .map_err(|_| TransactionParseError::InvalidAmount(amount_str.into()))?;
            Amount::from_f64(amount)
                .filter(|amount| amount >= &0.0)
                .ok_or_else(|| TransactionParseError::InvalidAmount(amount_str.into()))
        };
        // Make transaction
        let tx = match tx_type {
            "deposit" => Transaction::deposit(tx_id, amount()?),
            "withdrawal" => Transaction::withdrawal(tx_id, amount()?),
            "dispute" => Transaction::Dispute(tx_id),
            "resolve" => Transaction::resolution(ResolutionKind::Resolve, tx_id),
            "chargeback" => Transaction::resolution(ResolutionKind::Chargeback, tx_id),
            _ => {
                return Err(TransactionParseError::InvalidTransactionType(
                    tx_type.into(),
                ))
            }
        };
        Ok(ClientTransaction {
            client: client_id,
            tx,
        })
    }
}
