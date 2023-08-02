INSERT INTO categories
VALUES
    ('category-external-account', 'abc', 'Rent', 'For all my rent', 'ff3030', 0);

INSERT INTO subcategories
VALUES
    ('subcategory-external-account', 'abc', 'category-external-account', 'Test', '', 'ff3030');

INSERT INTO external_accounts
VALUES
    ('external-account-1', 'abc', 'Jumbo', 'The price it quite high', null, null),
    ('external-account-2', 'def', 'Jumbo', 'The price it quite high', null, null),
    ('external-account-3', 'abc', 'Evil landlord', 'The *******', 'category-external-account', 'subcategory-external-account');

INSERT INTO external_account_names
VALUES
    ('external-account-name-1', 'abc', 'EVIL_LAND_LORD_INC', 'external-account-3');
