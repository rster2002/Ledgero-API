INSERT INTO categories
VALUES
    ('category-external-account', 'abc', 'Rent', 'For all my rent', 'ff3030', 0);

INSERT INTO externalaccounts
VALUES
    ('external-account-1', 'abc', 'Jumbo', 'The price it quite high', null, null),
    ('external-account-2', 'def', 'Jumbo', 'The price it quite high', null, null),
    ('external-account-3', 'abc', 'Evil landlord', 'The *******', null, null);

INSERT INTO externalaccountnames
VALUES
    ('external-account-name-1', 'abc', 'EVIL_LAND_LORD_INC', 'external-account-3');
