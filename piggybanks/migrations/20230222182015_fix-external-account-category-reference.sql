ALTER TABLE ExternalAccounts
DROP CONSTRAINT externalaccounts_defaultcategoryid_fkey,
    ADD FOREIGN KEY (defaultcategoryid)
        references Categories (Id)
            on update cascade
            on delete set null;
