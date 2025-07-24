# Simple banking app
Has a state of the system in `State`. Whenever a transaction occurs it's send to the account it belongs to. In case the account doesn't exist it is created. The account process transaction double checks if the transaction belongs to this account (this is just in case the code is reused some place else, in this code it doesn't happen).

Disputes:
 - dispute for deposit works by locking the funds, moving them from available to held, the total amount doesn't change. In case of a chargeback the held(and total) amount is decreased and the account is locked.
 - dispute for withdrawal is slightly different, the amount is added to held, the available doesn't change. In case of a chargeback the held is decreased but the available is increase (the money was returned to the account), total doesn't change. The account is locked.

I've tested the code with the `sample.csv`. It includes all of the cases.
- Insufficient funds
- Transaction not in dispute
- Double dispute
- Resolve/chargeback for undisputed transaction
- Trying to dispute non existing transaction
- Sending transaction to a locked account

# Errors
The errors are propagated from the `state` to the main code, where they are printed. Custom TransactionError is used for this (using the `thiserror` crate).  Only errors that can panic the code are related to reading and writing the csv.

# Safety and robustness, Efficiency
I decided no to directly call the `State` functions, but instead I implemented channel for sending the transaction. This way if we decide to use several incoming streams, it can handle it. The only problem is if there would be too much data. There is only one stream so even unrelated transaction (to different accounts) are waiting for each other. But since the code for handling transaction is super simple this should not be an issue. It could happen if the code is more complex (e.g. reading a DB, or doing some cryptographic math on each transaction).
The file is not loaded at once, it's done line by line.
