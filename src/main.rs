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
use transaction::ClientTransaction;

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

/// Apply transactions parsed from a reader and apply each one to accounts
fn process_transaction_source<R>(source: R, accounts: &mut Accounts) -> Result<(), String>
where
    R: Read,
{
    for (i, line) in BufReader::new(source).lines().enumerate() {
        let line_no = i + 1;
        // Break on I/O error
        let line = line.map_err(|e| format!("Error reading line {}: {}", line_no, e))?;
        // Skip empty lines or header row if it is present
        if line.trim().is_empty() || i == 0 && line.trim().starts_with("type") {
            continue;
        }

        // Parse transaction
        let tx = line
            .parse::<ClientTransaction>()
            .map_err(|e| format!("Invalid transaction on line {}: {}", line_no, e))?;

        // Apply transaction
        if let Err(e) = accounts.transact(tx.clone()) {
            eprintln!("Error executing transaction on line {}: {}", line_no, e);
        }
    }
    Ok(())
}
