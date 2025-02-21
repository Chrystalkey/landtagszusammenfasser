-- This file should undo anything in `up.sql`
DROP EXTENSION pg_trgm;

DELETE FROM dokument;
DELETE FROM dokumententyp CASCADE;
DELETE FROM parlament CASCADE;
DELETE FROM stationstyp CASCADE;

DELETE FROM identifikatortyp CASCADE;
DELETE FROM vorgangstyp CASCADE;
