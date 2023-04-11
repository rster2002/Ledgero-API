ALTER TABLE Transactions
DROP CONSTRAINT transactions_bankaccountid_fkey,
ADD FOREIGN KEY (bankaccountid)
    references bankaccounts (id)
        on update cascade
        on delete no action;
