ALTER TABLE transactions
DROP CONSTRAINT transactions_bank_account_id_fkey,
ADD FOREIGN KEY (bank_account_id)
    references bank_accounts (id)
        on update cascade
        on delete no action;
