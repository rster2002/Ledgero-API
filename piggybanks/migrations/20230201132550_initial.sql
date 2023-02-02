create table Users
(
    Id           varchar(36) primary key not null,
    Username     varchar                 not null,
    PasswordHash varchar                 not null,

    CONSTRAINT username_unique UNIQUE (Username)
);
