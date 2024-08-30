CREATE TABLE
    data_thing_nullable (id SERIAl PRIMARY KEY, data VARCHAR(255));

CREATE TABLE
    primary_table (
        id SERIAl PRIMARY KEY,
        associated_data VARCHAR(255) NOT NULL,
        dt SERIAl REFERENCES data_thing_nullable (id) ON DELETE SET NULL
    );

CREATE TABLE
    secondary_table (id SERIAl PRIMARY KEY, 
    data VARCHAR(255));

CREATE TABLE
    list_data (
        id SERIAl PRIMARY KEY,
        data VARCHAR(255) NOT NULL,
        primtab SERIAl REFERENCES primary_table (id) ON DELETE CASCADE
    );

CREATE TABLE
    n_to_n (
        primtab SERIAl REFERENCES primary_table (id) ON DELETE CASCADE,
        sectab SERIAl REFERENCES secondary_table (id) ON DELETE CASCADE,
        data VARCHAR(255) NOT NULL
    );