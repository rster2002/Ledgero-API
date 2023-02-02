CREATE TABLE Categories
(
    Id          varchar(36) primary key not null,
    UserId      varchar(36)             not null
        references Users (Id)
            on update cascade
            on delete cascade,
    Name        varchar                 not null,
    Description varchar                 not null,
    HexColor    varchar                 not null
);

CREATE TABLE ExternalAccounts
(
    Id                varchar(36) primary key not null,
    UserId            varchar(36)             not null
        references Users (Id)
            on update cascade
            on delete cascade,
    Name              varchar                 not null,
    Description       varchar                 not null,
    DefaultCategoryId varchar(36)             null
        references Categories (Id)
            on update cascade
            on delete cascade
);

CREATE TABLE ExternalAccountNames
(
    Id                    varchar(36) primary key not null,
    UserId                varchar(36)             not null
        references Users (Id)
            on update cascade
            on delete cascade,
    Name                  varchar                 not null,
    ParentExternalAccount varchar(36)             not null
        references ExternalAccounts (Id)
            on update cascade
            on delete cascade
);

CREATE TABLE BankAccounts
(
    Id          varchar(36) primary key not null,
    IBAN        varchar                 not null,
    UserId      varchar(36)             not null
        references Users (Id)
            on update cascade
            on delete cascade,
    Name        varchar                 not null,
    Description varchar                 not null,
    HexColor    varchar                 not null,

    CONSTRAINT unique_iban UNIQUE (UserId, IBAN)
);

CREATE TABLE Transactions
(
    Id                  varchar(36) primary key not null,
    UserId              varchar(36)             not null
        references Users (Id)
            on update cascade
            on delete cascade,
    TransactionType     varchar                 not null,
    FollowNumber        varchar                 not null,
    Description         varchar                 not null,
    OriginalDescription varchar                 not null,
    CompleteAmount      bigint                  not null,
    Amount              bigint                  not null,
    Date                varchar                 not null,
    CategoryId          varchar(36)             null
        references Categories (Id)
            on update cascade
            on delete cascade,
    ParentTransactionId varchar(36)             null
        references Transactions (Id)
            on update cascade
            on delete cascade,
    ExternalAccountName varchar                 not null,
    ExternalAccountId   varchar(36)             null
        references ExternalAccounts (Id)
            on update cascade
            on delete cascade,
    BankAccountId       varchar(36)             not null
        references BankAccounts (Id)
            on update cascade
            on delete cascade,

    CONSTRAINT unique_follow_number UNIQUE (UserId, FollowNumber)
)
