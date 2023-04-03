ALTER TABLE externalaccounts
ADD COLUMN Image varchar null
    references blobs (token)
        on delete set null;

ALTER TABLE users
ADD COLUMN ProfileImage varchar null
    references blobs (token)
        on delete set null;

