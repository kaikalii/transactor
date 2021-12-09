# Description

Transactor is a simple command-line transaction simulator.

# Usage

Transactor reads in a CSV file passed as a command-line argument, executes the transactions it contains, and outputs CSV-formatted account balances to standard output.

There are 5 transaction types:
- deposit - add funds to an account
- withdrawal - withdraw funds from an account (if they are available)
- dispute - dispute a transaction, holding disputed funds (currently only deposits can be disputed)
- resolve - resolve a dispute by making the held funds available again
- chargback - resolve a dispute by removing the desputed held funds from the account and locking the account

## Example Input

Each line of the CSV file must start with the transaction type, followed by a client id and a transaction id. Deposits and withdrawals must then list a positive amount.

```
type, client, tx, amount
deposit, 1, 1, 20
deposit, 2, 2, 30
deposit, 3, 3, 50
withdrawal, 2, 4, 20
withdrawal, 2, 5, 20
deposit, 3, 6, 20
deposit, 3, 7, 40
dispute, 3, 6
dispute, 3, 7
resolve, 3, 6
chargeback, 3, 7
deposit, 4, 8, 100
deposit, 4, 9, 20
dispute, 4, 9
```

## Example Output

```
client,available,held,total,locked
1,20,0,20,false
2,10,0,10,false
3,70,0,70,true
4,100,20,120,false
```