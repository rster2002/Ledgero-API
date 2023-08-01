ALTER TABLE external_account_names
ADD UNIQUE (name, user_id),
ADD UNIQUE (id, name, parent_external_account, user_id);

ALTER TABLE transactions
ADD COLUMN external_account_name_id varchar(36) null,
ADD FOREIGN KEY (external_account_name_id, external_account_name, external_account_id, user_id)
    references external_account_names (id, name, parent_external_account, user_id);
