# Accounts

A simple toy payments engine

## Usage

```sh
# get help info
cargo run -- --help

# process transactions
cargo run -- inputs/transactions.csv > accounts.csv
```

## Comments

- I wasn't sure if disputes could happen on withdrawals but I tried to handle it anyways. The resulting code has no affect if this isn't the case so it was safe to add.
- I relied on a mixture of unit tests and type safety to ensure correctness. In the engine there are a couple more unit tests I could write to make sure errors are returned when expected and that certain conditions work as expected. I wrote the code first while testing against some sample input and then I filled in the tests. Normally, you'd want to write tests first but I was enjoying using the sample input. Thinking about how to fully test the core logic lead me to realize I could refactor the engine out of the main function.
- There is one sample input I was using which can be found in `inputs/transactions.csv`
- I added an extra check to make sure the dispute, resolve, and chargeback actions are being performed on a previous tx with the same client as the current tx. This may have been overkill but it seemed safer to add this check. Clients shouldn't be able to file disputes on other clients' transactions.
