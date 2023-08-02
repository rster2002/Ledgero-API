INSERT INTO imports
VALUES
    ('import-1', 'abc', '2023-02-11 11:00:00.000000 +00:00', 'bank-export.csv'),
    ('import-2', 'def', '2023-02-11 11:00:00.000000 +00:00', 'bank-export.csv');

INSERT INTO bank_accounts
VALUES
    ('bank-account-1', 'NL12 RABO 12345678910', 'abc', 'Primary bank account', 'For all of the normal stuff', 'ff3030'),
    ('bank-account-2', 'NL99 RABO 01987654321', 'def', 'Primary bank account', 'For all of the normal stuff', '3030ff');

INSERT INTO categories
VALUES
    ('transaction-category-1', 'abc', 'Groceries', 'For all the food', '303030', 1),
    ('transaction-category-2', 'def', 'Groceries', 'For all the food', '303030', 2);

INSERT INTO external_accounts
VALUES
    ('transaction-external-account-1', 'abc', 'Jumbo', 'The price it quite high', null, null, null, '303030'),
    ('transaction-external-account-2', 'def', 'Jumbo', 'The price it quite high', null, null, null, '303030');

INSERT INTO transactions
VALUES
    ('transaction-1', 'abc', 'transaction', '00000001', 'Salary', 'SALARY FROM WORK', 256700, 256700, '2023-02-11 11:00:00.000000 +00:00', null, null, 'Name of work', null, 'bank-account-1', 'import-1', null, 1),
    ('transaction-2', 'abc', 'transaction', '00000002', 'Groceries', 'Payment for Jumbo', -9300, -9300, '2023-02-11 11:00:00.000000 +00:00', 'transaction-category-1', null, 'Jumbo', 'transaction-external-account-1', 'bank-account-1', 'import-1', null, 1),
    ('transaction-3', 'abc', 'transaction', '00000003', 'rent', 'EVIL LAND LORD', -92000, -92000, '2023-02-11 11:00:00.000000 +00:00', null, null, 'EVIL_LAND_LORD_INC', null, 'bank-account-1', 'import-1', null, 1),
    ('transaction-4', 'def', 'transaction', '10000001', 'Salary', 'SALARY FROM WORK', 256700, 256700, '2023-02-11 11:00:00.000000 +00:00', null, null, 'Name of work', null, 'bank-account-2', 'import-2', null, 1),
    ('transaction-5', 'def', 'transaction', '20000002', 'Groceries', 'Payment for Jumbo', -9300, -9300, '2023-02-11 11:00:00.000000 +00:00', 'transaction-category-2', null, 'Jumbo', 'transaction-external-account-2', 'bank-account-2', 'import-2', null, 1),
    ('transaction-6', 'def', 'transaction', '30000003', 'rent', 'EVIL LAND LORD', -92000, -92000, '2023-02-11 11:00:00.000000 +00:00', null, null, 'EVIL_LAND_LORD_INC', null, 'bank-account-2', 'import-2', null, 1);
