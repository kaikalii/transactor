use crate::{account::Accounts, process_transaction_source};

#[test]
fn it_works() {
    let input = include_bytes!("../test.csv");

    let mut accounts = Accounts::default();

    process_transaction_source(input.as_slice(), &mut accounts).unwrap();

    assert_eq!(accounts[1].total(), 20.0);
    assert_eq!(accounts[2].total(), 10.0);
    assert_eq!(accounts[3].total(), 70.0);
}
