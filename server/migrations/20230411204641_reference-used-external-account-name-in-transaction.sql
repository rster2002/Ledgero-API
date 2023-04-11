ALTER TABLE externalaccountnames
DROP COLUMN id,
ADD PRIMARY KEY (userid, name, parentexternalaccount);

ALTER TABLE transactions
ADD COLUMN ExternalAccountNameEntry varchar(36) null,
ADD FOREIGN KEY (userid, externalaccountname, externalaccountid)
    references externalaccountnames (userid, name, parentexternalaccount)
        on delete set null;
