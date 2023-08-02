CREATE TABLE subcategories
(
    id             varchar(36) primary key not null,
    user_id         varchar(36)             not null
        references users (id)
            on update cascade
            on delete cascade,
    parent_category varchar(36)             not null
        references categories (id)
            on update cascade
            on delete cascade,
    name           varchar                 not null,
    description    varchar                 not null,
    hex_color       varchar                 not null
);

ALTER TABLE transactions
ADD COLUMN subcategory varchar(36) null
    references subcategories (id)
        on update cascade
        on delete set null;

ALTER TABLE external_accounts
ADD COLUMN default_subcategory_id varchar(36) null
    references subcategories (id)
        on update cascade
        on delete set null;
