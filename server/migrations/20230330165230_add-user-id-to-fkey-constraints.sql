-- Drop all foreign key constraints so the primary keys can be recreated.
ALTER TABLE transactions
    DROP CONSTRAINT transactions_bank_account_id_fkey,
    DROP CONSTRAINT transactions_category_id_fkey,
    DROP CONSTRAINT transactions_external_account_id_fkey,
    DROP CONSTRAINT transactions_parent_import_fkey,
    DROP CONSTRAINT transactions_parent_transaction_id_fkey,
    DROP CONSTRAINT transactions_subcategory_fkey;

ALTER TABLE subcategories
    DROP CONSTRAINT subcategories_parent_category_fkey;

ALTER TABLE skipped_transactions
    DROP CONSTRAINT skipped_transactions_import_id_fkey;

ALTER TABLE external_accounts
    DROP CONSTRAINT external_accounts_default_category_id_fkey,
    DROP CONSTRAINT external_accounts_default_subcategory_id_fkey,
    DROP CONSTRAINT external_accounts_image_fkey;

ALTER TABLE external_account_names
    DROP CONSTRAINT external_account_names_parent_external_account_fkey;

ALTER TABLE users
    DROP CONSTRAINT users_profile_image_fkey;

-- Recreate primary keys to include the user id.
ALTER TABLE bank_accounts
    DROP CONSTRAINT bank_accounts_pkey,
    ADD PRIMARY KEY (id, user_id);

ALTER TABLE categories
    DROP CONSTRAINT categories_pkey,
    ADD PRIMARY KEY (id, user_id);

ALTER TABLE blobs
    DROP CONSTRAINT blobs_pkey,
    ADD PRIMARY KEY (token, user_id);

ALTER TABLE external_account_names
    DROP CONSTRAINT external_account_names_pkey,
    ADD PRIMARY KEY (id, user_id);

ALTER TABLE external_accounts
    DROP CONSTRAINT external_accounts_pkey,
    ADD PRIMARY KEY (id, user_id);

ALTER TABLE grants
    DROP CONSTRAINT grants_pkey,
    ADD PRIMARY KEY (id, user_id);

ALTER TABLE imports
    DROP CONSTRAINT imports_pkey,
    ADD PRIMARY KEY (id, user_id);

ALTER TABLE subcategories
    DROP CONSTRAINT subcategories_pkey,
    ADD PRIMARY KEY (id, parent_category, user_id);

ALTER TABLE transactions
    DROP CONSTRAINT transactions_pkey,
    ADD PRIMARY KEY (id, user_id);

-- Recreate the foreign keys to include the user id.
ALTER TABLE transactions
    ADD FOREIGN KEY (category_id, user_id)
        references categories (id, user_id)
        on delete set null,
    ADD FOREIGN KEY (parent_transaction_id, user_id)
        references transactions (id, user_id)
        on delete cascade,
    ADD FOREIGN KEY (external_account_id, user_id)
        references external_accounts (id, user_id)
        on delete set null,
    ADD FOREIGN KEY (parent_import, user_id)
        references imports (id, user_id)
        on delete cascade,
    ADD FOREIGN KEY (subcategory_id, category_id, user_id)
        references subcategories (id, parent_category, user_id)
        on delete set null,
    ADD FOREIGN KEY (bank_account_id, user_id)
        references bank_accounts (id, user_id)
        on delete cascade;

ALTER TABLE subcategories
    ADD FOREIGN KEY (parent_category, user_id)
        references categories (id, user_id)
        on delete cascade;

ALTER TABLE skipped_transactions
    ADD FOREIGN KEY (import_id, user_id)
        references imports (id, user_id)
        on delete cascade;

ALTER TABLE external_accounts
    ADD FOREIGN KEY (default_category_id, user_id)
        references categories (id, user_id)
        on delete set null,
    ADD FOREIGN KEY (default_subcategory_id, default_category_id, user_id)
        references subcategories (id, parent_category, user_id)
        on delete set null,
    ADD FOREIGN KEY (image, user_id)
        references blobs (token, user_id)
        on delete set null;

ALTER TABLE external_account_names
    ADD FOREIGN KEY (parent_external_account, user_id)
        references external_accounts (id, user_id)
        on delete cascade;

ALTER TABLE users
    ADD FOREIGN KEY (profile_image, id)
        references blobs (token, user_id)
        on delete set null;
