use std::{
    collections::{HashMap, HashSet},
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
    pub fn transact(&mut self, id: TransactionId, ty: TransactionType) {
        match ty {
            TransactionType::Deposit(amount) => {
                self.balance += amount;
                self.history.insert(id, ty);
            }
            TransactionType::Withdrawal(amount) => {
                if !self.frozen && self.balance >= amount {
                    self.balance -= amount;
                }
                self.history.insert(id, ty);
            }
            TransactionType::Dispute => {
                if let Some(ty) = self.history.get(&id) {
                    match ty {
                        TransactionType::Deposit(amount) => {
                            self.balance -= *amount;
                            self.held += *amount;
                            self.disputed.insert(id);
                        }
                        TransactionType::Withdrawal(_) => {
                            self.disputed.insert(id);
                        }
                        _ => {}
                    }
                }
            }
            TransactionType::Resolve => {
                if self.disputed.remove(&id) {
                    if let Some(ty) = self.history.get(&id) {
                        match ty {
                            TransactionType::Deposit(amount) => {
                                self.balance += *amount;
                                self.held -= *amount;
                            }
                            TransactionType::Withdrawal(_) => {}
                            _ => {}
                        }
                    }
                }
            }
            TransactionType::Chargeback => {
                if self.disputed.remove(&id) {
                    if let Some(ty) = self.history.get(&id) {
                        match ty {
                            TransactionType::Deposit(amount) => {
                                self.held -= *amount;
                                self.frozen = true;
                            }
                            TransactionType::Withdrawal(amount) => {
                                self.balance += *amount;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

/// A collection of client accounts, indexed by client id
#[derive(Debug, Default)]
pub struct Accounts {
    accounts: HashMap<ClientId, Account>,
}

impl Accounts {
    /// Execute a transaction
    pub fn transact(&mut self, tx: Transaction) {
        self.accounts
            .entry(tx.client)
            .or_default()
            .transact(tx.id, tx.ty);
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
