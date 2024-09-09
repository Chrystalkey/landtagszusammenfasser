-- This file should undo anything in `up.sql`
GRANT ALL ON TABLE parlamente TO public;
GRANT ALL ON TABLE dokumenttypen TO public;
GRANT ALL ON TABLE gesetzeseigenschaften TO public;

DELETE FROM gesetzeseigenschaften WHERE TRUE;
DELETE FROM parlamente WHERE TRUE;
DELETE FROM dokumenttypen WHERE TRUE;
