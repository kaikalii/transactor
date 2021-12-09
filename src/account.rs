use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    ops::Index,
};

use crate::{
    amount::Amount,
    transaction::{ClientId, Transaction, TransactionId, TransactionType},
};

/// A client's account
#[derive(Debug, Default)]
pub struct Account {
    balance: Amount,
    held: Amount,
    frozen: bool,
    history: HashMap<TransactionId, TransactionType>,
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
    pub fn transact(
        &mut self,
        tx_id: TransactionId,
        ty: TransactionType,
    ) -> Result<(), TransactionError> {
        match ty {
            TransactionType::Deposit(amount) => {
                if self.history.contains_key(&tx_id) {
                    return Err(TransactionError::DuplicateTransactionId(tx_id));
                }
                self.balance += amount;
                self.history.insert(tx_id, ty);
            }
            TransactionType::Withdrawal(amount) => {
                if self.history.contains_key(&tx_id) {
                    return Err(TransactionError::DuplicateTransactionId(tx_id));
                }
                if self.frozen {
                    return Err(TransactionError::AccountFrozen);
                }
                if self.balance >= amount {
                    self.balance -= amount;
                    self.history.insert(tx_id, ty);
                } else {
                    return Err(TransactionError::InsufficentFunds {
                        current: self.balance,
                        requested: amount,
                    });
                }
            }
            TransactionType::Dispute => {
                if let Some(TransactionType::Deposit(amount)) = self.history.get(&tx_id) {
                    self.balance -= *amount;
                    self.held += *amount;
                    self.disputed.insert(tx_id);
                } else {
                    return Err(TransactionError::InvalidDispute);
                }
            }
            TransactionType::Resolve => {
                if self.disputed.remove(&tx_id) {
                    if let Some(TransactionType::Deposit(amount)) = self.history.get(&tx_id) {
                        self.balance += *amount;
                        self.held -= *amount;
                    }
                } else {
                    return Err(TransactionError::UndisputedResolve);
                }
            }
            TransactionType::Chargeback => {
                if self.disputed.remove(&tx_id) {
                    if let Some(TransactionType::Deposit(amount)) = self.history.get(&tx_id) {
                        self.held -= *amount;
                        self.frozen = true;
                        self.history.remove(&tx_id);
                    }
                } else {
                    return Err(TransactionError::UndisputedChargback);
                }
            }
        }
        Ok(())
    }
}

/// A collection of client accounts, indexed by client id
#[derive(Debug, Default)]
pub struct Accounts {
    accounts: HashMap<ClientId, Account>,
}

impl Accounts {
    /// Execute a transaction
    pub fn transact(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        self.accounts
            .entry(tx.client)
            .or_default()
            .transact(tx.id, tx.ty)
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

#[derive(Debug)]
pub enum TransactionError {
    AccountFrozen,
    InsufficentFunds { current: Amount, requested: Amount },
    InvalidDispute,
    UndisputedResolve,
    UndisputedChargback,
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
            TransactionError::InvalidDispute => write!(
                f,
                "The transaction of the given id does not exist or cannot be disputed"
            ),
            TransactionError::UndisputedResolve | TransactionError::UndisputedChargback => {
                write!(f, "The given transaction id was never disputed")
            }
            TransactionError::DuplicateTransactionId(id) => {
                write!(f, "Transaction id {} has already been used", id)
            }
        }
    }
}

impl Error for TransactionError {}
