INSERT INTO categories
VALUES
    ('category-A', 'abc', 'Category A', 'Test category A', '303030', 1),
    ('category-B', 'abc', 'Category B', 'Test category B', '303030', 2);

INSERT INTO subcategories
VALUES
    ('subcategory-A', 'abc', 'category-A', 'Subcategory A', 'Test subcategory', '030303'),
    ('subcategory-B', 'abc', 'category-B', 'Subcategory B', 'Test subcategory', '030303');

INSERT INTO imports
VALUES
    ('import-A', 'abc', '2023-02-11 11:00:00.000000 +00:00', 'bank-export.csv');

INSERT INTO bankaccounts
VALUES
    ('bank-account-A', 'NL12 RABO 12345678910', 'abc', 'Primary bank account', 'For all of the normal stuff', 'ff3030');

INSERT INTO transactions
VALUES
    ('move-transaction-1', 'abc', 'transaction', '00000001', 'Salary', 'SALARY FROM WORK', 1000, 1000, '2023-02-11 11:00:00.000000 +00:00', 'category-A', null, 'Name of work', null, 'bank-account-A', 'import-A', 'subcategory-A', 1),
    ('move-transaction-2', 'abc', 'transaction', '00000002', 'Groceries', 'Payment for Jumbo', 2000, 2000, '2023-02-11 11:00:00.000000 +00:00', 'category-B', null, 'Jumbo', null, 'bank-account-A', 'import-A', 'subcategory-B', 2);
