CREATE TABLE Subcategories
(
    Id             varchar(36) primary key not null,
    UserId         varchar(36)             not null
        references Users (Id)
            on update cascade
            on delete cascade,
    ParentCategory varchar(36)             not null
        references Categories (Id)
            on update cascade
            on delete cascade,
    Name           varchar                 not null,
    Description    varchar                 not null,
    HexColor       varchar                 not null
);

ALTER TABLE Transactions
ADD COLUMN Subcategory varchar(36) null
    references Subcategories (Id)
        on update cascade
        on delete set null;

ALTER TABLE ExternalAccounts
ADD COLUMN DefaultSubcategoryId varchar(36) null
    references Subcategories (Id)
        on update cascade
        on delete set null;
