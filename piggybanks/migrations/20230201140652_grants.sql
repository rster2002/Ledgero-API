create table Grants
(
    Id       varchar primary key not null,
    UserId   varchar             not null
        references users(id)
            on update cascade
            on delete cascade,
    ExpireAt varchar             not null
);
