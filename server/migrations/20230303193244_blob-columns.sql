ALTER TABLE external_accounts
ADD COLUMN image varchar null
    references blobs (token)
        on delete set null;

ALTER TABLE users
ADD COLUMN profile_image varchar null
    references blobs (token)
        on delete set null;

