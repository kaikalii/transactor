mod account;
mod amount;
#[cfg(test)]
mod test;
mod transaction;

use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Read},
    process::exit,
};

use account::Accounts;
use transaction::Transaction;

fn main() {
    // Get the input file path
    let input_path = if let Some(path) = env::args().nth(1) {
        path
    } else {
        eprintln!("Expected input file path");
        exit(1);
    };

    // Open the input file
    let input_file = match File::open(&input_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Unable to open {:?}: {}", input_path, e);
            return;
        }
    };

    // Initialize accounts
    let mut accounts = Accounts::default();

    // Process all transactions from file
    if let Err(e) = process_transaction_source(input_file, &mut accounts) {
        eprintln!("{}", e);
        exit(1);
    }

    // Output account data on stdout
    println!("client,available,held,total,locked");
    for (client_id, account) in accounts.iter() {
        println!(
            "{},{},{},{},{}",
            client_id,
            account.balance(),
            account.held(),
            account.total(),
            account.is_frozen()
        );
    }
}

fn process_transaction_source<R>(source: R, accounts: &mut Accounts) -> Result<(), String>
where
    R: Read,
{
    for (i, line) in BufReader::new(source).lines().enumerate() {
        // Break on I/O error
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                return Err(format!("Error reading line {}: {}", i, e));
            }
        };
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse transaction
        let transaction = match line.parse::<Transaction>() {
            Ok(tx) => tx,
            Err(e) => {
                return Err(format!("Invalid transaction on line {}: {}", i + 1, e));
            }
        };

        // Apply transaction
        accounts.transact(transaction);
    }
    Ok(())
}
