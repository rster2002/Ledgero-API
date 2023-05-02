ALTER TABLE transactions
ADD COLUMN RelatedMoveTransaction varchar(36) null,
ADD FOREIGN KEY (userid, RelatedMoveTransaction)
    references transactions (userid, id)
        on delete cascade
        initially deferred;
