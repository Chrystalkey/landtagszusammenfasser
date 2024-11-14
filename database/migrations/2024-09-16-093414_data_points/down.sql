-- This file should undo anything in `up.sql`
GRANT ALL ON TABLE gesetzestyp TO public;
GRANT ALL ON TABLE parlament TO public;
GRANT ALL ON TABLE dokumenttyp TO public;

-- DELETE FROM gesetzestyp WHERE TRUE;
DELETE FROM parlament WHERE TRUE;
DELETE FROM dokumenttyp WHERE TRUE;
DELETE FROM stationstyp WHERE TRUE;
