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

    assert_eq!(accounts[1].total(), 18.3);
    assert_eq!(accounts[2].total(), 10.1235);
    assert_eq!(accounts[3].total(), 70.0);
    assert_eq!(accounts[4].balance(), 100.0);
    assert_eq!(accounts[4].held(), 20.6);
    assert_eq!(accounts[4].total(), 120.6);
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
        .transact(Transaction::withdrawal(1, Amount::from_f64(55.5).unwrap()))
        .unwrap();
    assert_eq!(account.total(), 44.5);
    account
        .transact(Transaction::withdrawal(1, Amount::from_f64(60.0).unwrap()))
        .unwrap_err();
    assert_eq!(account.total(), 44.5);
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

#[test]
fn amount_reliability() {
    // Float arithmetic can accumulate errors
    let mut i = 0.0;
    let delta = 0.3;
    i += delta;
    i += delta;
    i += delta;
    assert_ne!(i, 0.9);

    // Amount arithmetic cannot
    let mut i = Amount::from_f64(0.0).unwrap();
    let delta = Amount::from_f64(0.3).unwrap();
    i += delta;
    i += delta;
    i += delta;
    assert_eq!(i, 0.9);
}
