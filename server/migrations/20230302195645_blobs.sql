CREATE TABLE blobs
(
    token       varchar     not null primary key, -- This is NOT a uuid
    user_id      varchar(36) not null
        references users (Id)
            on update cascade
            on delete cascade,
    mime_type    varchar(32) not null,
    uploaded_at  timestamptz not null,
    confirmed_at timestamptz null
);
