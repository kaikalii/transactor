//! Types for working with client accounts

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    ops::Index,
};

use crate::{amount::Amount, transaction::*};

/// A client's account
#[derive(Debug, Default)]
pub struct Account {
    balance: Amount,
    held: Amount,
    frozen: bool,
    history: HashMap<TransactionId, BalanceChange>,
    disputed: HashSet<TransactionId>,
}

// `Account`' fields are behind getters because they should only be modifiable through transactions
impl Account {
    /// Get the account's currently accessible balance
    pub fn balance(&self) -> Amount {
        self.balance
    }
    /// Get the account's currently held balance
    pub fn held(&self) -> Amount {
        self.held
    }
    /// Check whether the account is frozen
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }
    /// Get the account's total balance
    pub fn total(&self) -> Amount {
        self.balance + self.held
    }
    /// Execute a transaction on the account
    pub fn transact(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        match tx {
            Transaction::Change { tx_id, change } => {
                if self.history.contains_key(&tx_id) {
                    return Err(TransactionError::DuplicateTransactionId(tx_id));
                }
                match change.kind {
                    ChangeKind::Deposit => self.balance += change.amount,
                    ChangeKind::Withdrawal => {
                        // Prevent frozen accounts from being withdrawn from
                        if self.frozen {
                            return Err(TransactionError::AccountFrozen);
                        }
                        // Ensure the funds are available
                        if self.balance >= change.amount {
                            self.balance -= change.amount;
                        } else {
                            return Err(TransactionError::InsufficentFunds {
                                current: self.balance,
                                requested: change.amount,
                            });
                        }
                    }
                }
                self.history.insert(tx_id, change);
            }
            Transaction::Dispute(tx_id) => {
                // When initiating a dispute, put disputed funds into holding
                if let Some(BalanceChange {
                    kind: ChangeKind::Deposit,
                    amount,
                }) = self.history.get(&tx_id)
                {
                    self.balance -= *amount;
                    self.held += *amount;
                    self.disputed.insert(tx_id);
                } else {
                    return Err(TransactionError::InvalidDispute(tx_id));
                }
            }
            Transaction::Resolution { kind, tx_id } => {
                if self.disputed.remove(&tx_id) {
                    if let Some(BalanceChange {
                        kind: ChangeKind::Deposit,
                        amount,
                    }) = self.history.get(&tx_id)
                    {
                        match kind {
                            ResolutionKind::Resolve => {
                                // When resolving a disputed deposit, make disputed held funds available again
                                self.balance += *amount;
                                self.held -= *amount;
                            }
                            ResolutionKind::Chargeback => {
                                // When charging back a dispute, remove the held funds and freeze the account
                                self.held -= *amount;
                                self.frozen = true;
                                // The transaction is removed from the history so it
                                // cannot be disputed and charged back again
                                self.history.remove(&tx_id);
                            }
                        }
                    }
                } else {
                    return Err(TransactionError::UndisputedResolution { tx_id, kind });
                }
            }
        }
        Ok(())
    }
}

/// A collection of client [`Account`]s, indexed by client id
#[derive(Debug, Default)]
pub struct Accounts {
    accounts: HashMap<ClientId, Account>,
}

impl Accounts {
    /// Execute a transaction
    pub fn transact(&mut self, client_tx: ClientTransaction) -> Result<(), TransactionError> {
        self.accounts
            .entry(client_tx.client)
            .or_default()
            .transact(client_tx.tx)
    }
    /// Iterate over all accounts and their client ids
    pub fn iter(&self) -> impl Iterator<Item = (ClientId, &Account)> {
        self.accounts.iter().map(|(&id, account)| (id, account))
    }
    /// Get the account associated with the given client id
    pub fn get(&self, client_id: ClientId) -> Option<&Account> {
        self.accounts.get(&client_id)
    }
}

impl Index<ClientId> for Accounts {
    type Output = Account;
    fn index(&self, id: ClientId) -> &Self::Output {
        self.get(id)
            .unwrap_or_else(|| panic!("Invalid client id: {}", id))
    }
}

/// An error that can occur when executing a transaction
#[derive(Debug)]
pub enum TransactionError {
    AccountFrozen,
    InsufficentFunds {
        current: Amount,
        requested: Amount,
    },
    InvalidDispute(TransactionId),
    UndisputedResolution {
        tx_id: TransactionId,
        kind: ResolutionKind,
    },
    DuplicateTransactionId(TransactionId),
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::AccountFrozen => write!(f, "Account is frozen"),
            TransactionError::InsufficentFunds { current, requested } => write!(
                f,
                "Attempted to withdraw {} from an account with {} avaiable",
                requested, current
            ),
            TransactionError::InvalidDispute(tx_id) => write!(
                f,
                "The transaction with id {} does not exist or cannot be disputed",
                tx_id
            ),
            TransactionError::UndisputedResolution { tx_id, .. } => {
                write!(f, "A transaction with id {} was never disputed", tx_id)
            }
            TransactionError::DuplicateTransactionId(id) => {
                write!(f, "Transaction id {} has already been used", id)
            }
        }
    }
}

impl Error for TransactionError {}
