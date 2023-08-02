create table users
(
    id           varchar(36) primary key not null,
    username     varchar                 not null,
    password_hash varchar                 not null,

    CONSTRAINT username_unique UNIQUE (username)
);
