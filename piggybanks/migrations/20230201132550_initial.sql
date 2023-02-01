create table Users
(
    Id           varchar primary key not null,
    Username     varchar             not null,
    PasswordHash varchar             not null,

    CONSTRAINT username_unique UNIQUE (Username)
);
