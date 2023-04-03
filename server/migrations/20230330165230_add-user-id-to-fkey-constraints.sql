-- Drop all foreign key constraints so the primary keys can be recreated.
ALTER TABLE transactions
    DROP CONSTRAINT transactions_bankaccountid_fkey,
    DROP CONSTRAINT transactions_categoryid_fkey,
    DROP CONSTRAINT transactions_externalaccountid_fkey,
    DROP CONSTRAINT transactions_parentimport_fkey,
    DROP CONSTRAINT transactions_parenttransactionid_fkey,
    DROP CONSTRAINT transactions_subcategory_fkey;

ALTER TABLE subcategories
    DROP CONSTRAINT subcategories_parentcategory_fkey;

ALTER TABLE skippedtransactions
    DROP CONSTRAINT skippedtransactions_importid_fkey;

ALTER TABLE externalaccounts
    DROP CONSTRAINT externalaccounts_defaultcategoryid_fkey,
    DROP CONSTRAINT externalaccounts_defaultsubcategoryid_fkey,
    DROP CONSTRAINT externalaccounts_image_fkey;

ALTER TABLE externalaccountnames
    DROP CONSTRAINT externalaccountnames_parentexternalaccount_fkey;

ALTER TABLE users
    DROP CONSTRAINT users_profileimage_fkey;

-- Recreate primary keys to include the user id.
ALTER TABLE bankaccounts
    DROP CONSTRAINT bankaccounts_pkey,
    ADD PRIMARY KEY (id, userid);

ALTER TABLE categories
    DROP CONSTRAINT categories_pkey,
    ADD PRIMARY KEY (id, userid);

ALTER TABLE blobs
    DROP CONSTRAINT blobs_pkey,
    ADD PRIMARY KEY (token, userid);

ALTER TABLE externalaccountnames
    DROP CONSTRAINT externalaccountnames_pkey,
    ADD PRIMARY KEY (id, userid);

ALTER TABLE externalaccounts
    DROP CONSTRAINT externalaccounts_pkey,
    ADD PRIMARY KEY (id, userid);

ALTER TABLE grants
    DROP CONSTRAINT grants_pkey,
    ADD PRIMARY KEY (id, userid);

ALTER TABLE imports
    DROP CONSTRAINT imports_pkey,
    ADD PRIMARY KEY (id, userid);

ALTER TABLE subcategories
    DROP CONSTRAINT subcategories_pkey,
    ADD PRIMARY KEY (id, parentcategory, userid);

ALTER TABLE transactions
    DROP CONSTRAINT transactions_pkey,
    ADD PRIMARY KEY (id, userid);

-- Recreate the foreign keys to include the user id.
ALTER TABLE transactions
    ADD FOREIGN KEY (categoryid, userid)
        references categories (id, userid)
        on delete set null,
    ADD FOREIGN KEY (parenttransactionid, userid)
        references transactions (id, userid)
        on delete cascade,
    ADD FOREIGN KEY (externalaccountid, userid)
        references externalaccounts (id, userid)
        on delete set null,
    ADD FOREIGN KEY (parentimport, userid)
        references imports (id, userid)
        on delete cascade,
    ADD FOREIGN KEY (subcategoryid, categoryid, userid)
        references subcategories (id, parentcategory, userid)
        on delete set null,
    ADD FOREIGN KEY (bankaccountid, userid)
        references bankaccounts (id, userid)
        on delete cascade;

ALTER TABLE subcategories
    ADD FOREIGN KEY (parentcategory, userid)
        references categories (id, userid)
        on delete cascade;

ALTER TABLE skippedtransactions
    ADD FOREIGN KEY (importid, userid)
        references imports (id, userid)
        on delete cascade;

ALTER TABLE externalaccounts
    ADD FOREIGN KEY (defaultcategoryid, userid)
        references categories (id, userid)
        on delete set null,
    ADD FOREIGN KEY (defaultsubcategoryid, defaultcategoryid, userid)
        references subcategories (id, parentcategory, userid)
        on delete set null,
    ADD FOREIGN KEY (image, userid)
        references blobs (token, userid)
        on delete set null;

ALTER TABLE externalaccountnames
    ADD FOREIGN KEY (parentexternalaccount, userid)
        references externalaccounts (id, userid)
        on delete cascade;

ALTER TABLE users
    ADD FOREIGN KEY (profileimage, id)
        references blobs (token, userid)
        on delete set null;
