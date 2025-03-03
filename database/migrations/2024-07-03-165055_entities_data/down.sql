DROP TRIGGER trg_delete_orphaned_dokument_stellungnahme ON stellungnahme;
DROP FUNCTION delete_orphaned_dokument_stellungnahme;

DROP TRIGGER trg_delete_orphaned_dokument_station ON rel_station_dokument;
DROP FUNCTION delete_orphaned_dokument_station;

DROP TABLE stellungnahme;

DROP TABLE rel_station_schlagwort;
DROP TABLE rel_station_dokument;
DROP TABLE rel_station_gesetz;
DROP TABLE station;

DROP TABLE rel_gsvh_links;
DROP TABLE rel_gsvh_id;
DROP TABLE rel_gsvh_init_person;
DROP TABLE rel_gsvh_init;
DROP TABLE gesetzesvorhaben;

DROP TABLE rel_dok_schlagwort;
DROP TABLE rel_dok_autorperson;
DROP TABLE rel_dok_autor;
DROP TABLE dokument;