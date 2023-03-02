CREATE TABLE Blobs
(
    Token       varchar(32) not null primary key, -- This is NOT a uuid
    UserId      varchar(36) not null
        references Users (Id)
            on update cascade
            on delete cascade,
    MimeType    varchar(32) not null,
    ConfirmedAt timestamptz not null
);
