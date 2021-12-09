use crate::{
    account::{Account, Accounts},
    amount::Amount,
    process_transaction_source,
    transaction::{DisputeKind, Transaction},
};

#[test]
fn it_works() {
    let input = include_bytes!("../test.csv");

    let mut accounts = Accounts::default();

    process_transaction_source(input.as_slice(), &mut accounts).unwrap();

    assert_eq!(accounts[1].total(), 20.0);
    assert_eq!(accounts[2].total(), 10.0);
    assert_eq!(accounts[3].total(), 70.0);
    assert_eq!(accounts[4].balance(), 100.0);
    assert_eq!(accounts[4].held(), 20.0);
}

fn account_with_100() -> Account {
    let mut account = Account::default();
    account
        .transact(Transaction::deposit(0, Amount::from_f64(100.0).unwrap()))
        .unwrap();
    account
}

#[test]
fn deposit() {
    let account = account_with_100();
    assert_eq!(account.total(), 100.0);
}

#[test]
fn withdrawal() {
    let mut account = account_with_100();
    account
        .transact(Transaction::withdrawal(1, Amount::from_f64(60.0).unwrap()))
        .unwrap();
    assert_eq!(account.total(), 40.0);
    account
        .transact(Transaction::withdrawal(1, Amount::from_f64(60.0).unwrap()))
        .unwrap_err();
    assert_eq!(account.total(), 40.0);
}

#[test]
fn resolve() {
    let mut account = account_with_100();
    account
        .transact(Transaction::dispute(DisputeKind::Initiate, 0))
        .unwrap();
    assert_eq!(account.balance(), 0.0);
    assert_eq!(account.held(), 100.0);
    account
        .transact(Transaction::dispute(DisputeKind::Resolve, 0))
        .unwrap();
    assert_eq!(account.balance(), 100.0);
    assert_eq!(account.held(), 0.0);
    assert!(!account.is_frozen());
}

#[test]
fn chargeback() {
    let mut account = account_with_100();
    account
        .transact(Transaction::dispute(DisputeKind::Initiate, 0))
        .unwrap();
    assert_eq!(account.balance(), 0.0);
    assert_eq!(account.held(), 100.0);
    account
        .transact(Transaction::dispute(DisputeKind::Chargeback, 0))
        .unwrap();
    assert_eq!(account.balance(), 0.0);
    assert_eq!(account.held(), 0.0);
    assert!(account.is_frozen());
}

#[test]
fn double_chargeback() {
    let mut account = account_with_100();
    account
        .transact(Transaction::dispute(DisputeKind::Initiate, 0))
        .unwrap();
    assert_eq!(account.balance(), 0.0);
    assert_eq!(account.held(), 100.0);
    account
        .transact(Transaction::dispute(DisputeKind::Chargeback, 0))
        .unwrap();
    assert_eq!(account.balance(), 0.0);
    assert_eq!(account.held(), 0.0);
    assert!(account.is_frozen());
    account
        .transact(Transaction::dispute(DisputeKind::Chargeback, 0))
        .unwrap_err();
    assert_eq!(account.balance(), 0.0);
    assert_eq!(account.held(), 0.0);
}
