CREATE TABLE Categories
(
    Id          varchar primary key not null,
    UserId      varchar             not null
        references Users (Id)
            on update cascade
            on delete cascade,
    Name        varchar             not null,
    Description varchar             not null,
    HexColor    varchar             not null
);

CREATE TABLE ExternalAccounts
(
    Id              varchar primary key not null,
    UserId          varchar             not null
        references Users (Id)
            on update cascade
            on delete cascade,
    Name            varchar             not null,
    Description     varchar             not null,
    DefaultCategory varchar             null
        references Categories (Id)
            on update cascade
            on delete cascade
);

CREATE TABLE ExternalAccountNames
(
    Id                    varchar primary key not null,
    UserId                varchar             not null
        references Users (Id)
            on update cascade
            on delete cascade,
    Name                  varchar             not null,
    ParentExternalAccount varchar             not null
        references ExternalAccounts (Id)
            on update cascade
            on delete cascade
);

CREATE TABLE Transactions
(
    Id                  varchar primary key not null,
    UserId              varchar             not null
        references Users (Id)
            on update cascade
            on delete cascade,
    TransactionType     varchar             not null,
    FollowNumber        varchar             not null,
    Description         varchar             not null,
    CompleteAmount      bigint              not null,
    Amount              bigint              not null,
    CategoryId          varchar             null
        references Categories (Id)
            on update cascade
            on delete cascade,
    ParentTransaction   varchar             null
        references Transactions (Id)
            on update cascade
            on delete cascade,
    ExternalAccountName varchar             not null,
    ExternalAccountId   varchar             null
        references ExternalAccounts (Id)
            on update cascade
            on delete cascade,

    CONSTRAINT unique_follow_number UNIQUE (FollowNumber)
)
