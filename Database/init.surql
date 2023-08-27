REMOVE TABLE Doodles;

DEFINE TABLE Doodles SCHEMAFULL;

DEFINE FIELD name ON TABLE Doodles TYPE string
    ASSERT $value != NONE;
DEFINE FIELD description ON TABLE Doodles TYPE string
    ASSERT $value != NONE;

