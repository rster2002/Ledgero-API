ALTER TABLE transactions
ADD COLUMN related_move_transaction varchar(36) null,
ADD FOREIGN KEY (user_id, related_move_transaction)
    references transactions (user_id, id)
        on delete cascade
        initially deferred;
