create table Grants
(
    Id       varchar(36) primary key not null,
    UserId   varchar(36)             not null
        references users (id)
            on update cascade
            on delete cascade,
    ExpireAt varchar                 not null
);
