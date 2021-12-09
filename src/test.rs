use crate::{
    account::{Account, Accounts},
    amount::Amount,
    process_transaction_source,
    transaction::TransactionType,
};

#[test]
fn it_works() {
    let input = include_bytes!("../test.csv");

    let mut accounts = Accounts::default();

    process_transaction_source(input.as_slice(), &mut accounts).unwrap();

    assert_eq!(accounts[1].total(), 20.0);
    assert_eq!(accounts[2].total(), 10.0);
    assert_eq!(accounts[3].total(), 70.0);
    assert_eq!(accounts[4].total(), 100.0);
}

fn account_with_100() -> Account {
    let mut account = Account::default();
    account
        .transact(
            0,
            TransactionType::Deposit(Amount::from_f64(100.0).unwrap()),
        )
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
    let withdraw_60 = TransactionType::Withdrawal(Amount::from_f64(60.0).unwrap());
    account.transact(1, withdraw_60).unwrap();
    account.transact(2, withdraw_60).unwrap_err();
    assert_eq!(account.total(), 40.0);
}
