INSERT INTO transactions
VALUES
    ('split-1', 'abc', 'split', '999999990', 'Allocated 1', 'Allocated', 100000, 100000, '2023-02-11 11:00:00.000000 +00:00', null, 'transaction-1', 'Name of work', null, 'bank-account-1', null, null, 3),
    ('split-2', 'abc', 'split', '999999991', 'Allocated 2', 'Allocated', 50000, 50000, '2023-02-11 11:00:00.000000 +00:00', null, 'transaction-1', 'Name of work', null, 'bank-account-1', null, null, 4),
    ('split-3', 'abc', 'split', '999999994', 'Allocated 3', 'Allocated', -3000, -3000, '2023-02-11 11:00:00.000000 +00:00', null, 'transaction-2', 'Name of work', null, 'bank-account-1', null, null, 5);

UPDATE transactions
SET amount = 106700
WHERE id = 'transaction-1';
