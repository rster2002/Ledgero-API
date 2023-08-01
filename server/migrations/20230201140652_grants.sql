create table grants
(
    id       varchar(36) primary key not null,
    user_id   varchar(36)             not null
        references users (id)
            on update cascade
            on delete cascade,
    expire_at varchar                 not null
);
