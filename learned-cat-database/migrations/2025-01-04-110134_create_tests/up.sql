-- Your SQL goes here
CREATE TABLE User (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL
);

CREATE TABLE Test (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    caption VARCHAR NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE Variant (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    test_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    mark FLOAT NOT NULL,
    begin_timestamp VARCHAR NOT NULL
);
