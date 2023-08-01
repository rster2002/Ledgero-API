CREATE TABLE categories
(
    id          varchar(36) primary key not null,
    user_id     varchar(36)             not null
        references users (Id)
            on update cascade
            on delete cascade,
    name        varchar                 not null,
    description varchar                 not null,
    hex_color   varchar                 not null
);

CREATE TABLE external_accounts
(
    id                  varchar(36) primary key not null,
    user_id             varchar(36)             not null
        references users (id)
            on update cascade
            on delete cascade,
    name                varchar                 not null,
    description         varchar                 not null,
    default_category_id varchar(36)             null
        references categories (id)
            on update cascade
            on delete cascade
);

CREATE TABLE external_account_names
(
    id                      varchar(36) primary key not null,
    user_id                 varchar(36)             not null
        references users (id)
            on update cascade
            on delete cascade,
    name                    varchar                 not null,
    parent_external_account varchar(36)             not null
        references external_accounts (id)
            on update cascade
            on delete cascade
);

CREATE TABLE bank_accounts
(
    id          varchar(36) primary key not null,
    iban        varchar                 not null,
    user_id     varchar(36)             not null
        references users (id)
            on update cascade
            on delete cascade,
    name        varchar                 not null,
    description varchar                 not null,
    hex_color   varchar                 not null,

    CONSTRAINT unique_iban UNIQUE (user_id, iban)
);

CREATE TABLE imports
(
    id          varchar(36) primary key  not null,
    user_id     varchar(36)              not null
        references users (id)
            on update cascade
            on delete cascade,
    imported_at timestamp with time zone not null,
    file_name   varchar                  not null
);

CREATE TABLE transactions
(
    id                    varchar(36) primary key  not null,
    user_id               varchar(36)              not null
        references users (id)
            on update cascade
            on delete cascade,
    transaction_type      varchar                  not null,
    follow_number         varchar                  not null,
    description           varchar                  not null,
    original_description  varchar                  not null,
    complete_amount       bigint                   not null,
    amount                bigint                   not null,
    date                  timestamp with time zone not null,
    category_id           varchar(36)              null
        references categories (id)
            on update cascade
            on delete set null,
    parent_transaction_id varchar(36)              null
        references transactions (id)
            on update cascade
            on delete cascade,
    external_account_name varchar                  not null,
    external_account_id   varchar(36)              null
        references external_accounts (id)
            on update cascade
            on delete set null,
    bank_account_id       varchar(36)              not null
        references bank_accounts (id)
            on update cascade
            on delete cascade,
    parent_import         varchar(36)              null
        references Imports (id)
            on update cascade
            on delete cascade,

    CONSTRAINT unique_follow_number UNIQUE (user_id, follow_number)
);

CREATE TABLE skipped_transactions
(
    import_id     varchar(36) not null
        references imports (id)
            on update cascade
            on delete cascade,
    user_id       varchar(36) not null,
    follow_number varchar(36) not null,

    primary key (import_id, user_id, follow_number),
    foreign key (user_id, follow_number)
        references transactions (user_id, follow_number)
        on update cascade
        on delete cascade
);
