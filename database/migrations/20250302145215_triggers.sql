CREATE OR REPLACE FUNCTION dokref_stat_tops_stln()
RETURNS TRIGGER LANGUAGE plpgsql AS $$ BEGIN
    -- Check if the dokument is still referenced in either table
    IF NOT EXISTS (
        SELECT 1 FROM rel_station_dokument WHERE dok_id = OLD.id
    ) AND NOT EXISTS (
        SELECT 1 FROM tops_doks WHERE dok_id = OLD.id
    ) AND NOT EXISTS (
        SELECT 1 FROM rel_station_stln WHERE dok_id = OLD.id
    ) THEN
        -- Delete the dokument if no references exist
        DELETE FROM dokument WHERE id = OLD.id;
    END IF;
    
    RETURN NULL; -- AFTER trigger should return NULL
END;
$$;

CREATE TRIGGER trg_check_dokument_after_station_change
AFTER DELETE OR UPDATE OF dok_id ON rel_station_dokument
FOR EACH ROW
EXECUTE PROCEDURE dokref_stat_tops_stln();

CREATE TRIGGER trg_check_dokument_after_tops_change
AFTER DELETE OR UPDATE OF dok_id ON tops_doks
FOR EACH ROW
EXECUTE PROCEDURE dokref_stat_tops_stln();

CREATE TRIGGER trg_check_dokument_after_stln_change
AFTER DELETE OR UPDATE OF dok_id ON rel_station_stln
FOR EACH ROW
EXECUTE PROCEDURE dokref_stat_tops_stln();
-----------------------------------------------
CREATE OR REPLACE FUNCTION dokref_sitzung()
RETURNS TRIGGER LANGUAGE plpgsql AS $$ BEGIN
    -- Check if the dokument is still referenced in either table
    IF NOT EXISTS (
        SELECT 1 FROM rel_sitzung_doks WHERE did = OLD.did
    ) THEN
        -- Delete the dokument if no references exist
        DELETE FROM dokument WHERE id = OLD.did;
    END IF;
    
    RETURN NULL; -- AFTER trigger should return NULL
END;
$$;

CREATE TRIGGER trg_check_dokument_after_sitzung_change
AFTER DELETE OR UPDATE OF did ON rel_sitzung_doks
FOR EACH ROW
EXECUTE PROCEDURE dokref_sitzung();
-----------------------------------------------
CREATE OR REPLACE FUNCTION autorref_experte()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    -- Check if the autor is still referenced in either table
    IF NOT EXISTS (
        SELECT 1 FROM rel_sitzung_experten WHERE eid = OLD.eid
    ) THEN DELETE FROM autor WHERE id = OLD.id;
    END IF;
    
    RETURN NULL; -- AFTER trigger should return NULL
END;
$$;

CREATE TRIGGER trg_check_autor_after_experten_change
AFTER DELETE OR UPDATE OF eid ON rel_sitzung_experten
FOR EACH ROW
EXECUTE PROCEDURE autorref_experte();
-----------------------------------------------
CREATE OR REPLACE FUNCTION autorref_initiator()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    -- Check if the autor is still referenced in either table
    IF NOT EXISTS (
        SELECT 1 FROM rel_vorgang_init WHERE in_id= OLD.in_id
    ) THEN DELETE FROM autor WHERE id = OLD.in_id;
    END IF;
    RETURN NULL; -- AFTER trigger should return NULL
END;
$$;

CREATE TRIGGER trg_check_autor_after_init_change
AFTER DELETE OR UPDATE OF in_id ON rel_vorgang_init
FOR EACH ROW
EXECUTE PROCEDURE autorref_initiator();
-----------------------------------------------
CREATE OR REPLACE FUNCTION autorref_dokument()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    -- Check if the autor is still referenced in either table
    IF NOT EXISTS (
        SELECT 1 FROM rel_dok_autor WHERE aut_id= OLD.aut_id
    ) THEN DELETE FROM autor WHERE id = OLD.aut_id;
    END IF;
    RETURN NULL; -- AFTER trigger should return NULL
END;
$$;

CREATE TRIGGER trg_check_autor_after_dokumentchange
AFTER DELETE OR UPDATE OF aut_id ON rel_dok_autor
FOR EACH ROW
EXECUTE PROCEDURE autorref_dokument();

