ALTER TABLE externalaccountnames
ADD UNIQUE (name, userid),
ADD UNIQUE (id, name, ParentExternalAccount, UserId);

ALTER TABLE transactions
ADD COLUMN ExternalAccountNameId varchar(36) null,
ADD FOREIGN KEY (ExternalAccountNameId, ExternalAccountName, ExternalAccountId, UserId)
    references ExternalAccountNames (id, name, ParentExternalAccount, UserId);
