ALTER TABLE external_accounts
DROP CONSTRAINT external_accounts_default_category_id_fkey,
    ADD FOREIGN KEY (default_category_id)
        references categories (id)
            on update cascade
            on delete set null;
