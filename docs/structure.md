# Project Ideas
## Structure
The project consists of three parts:
- Webserver
    - Retrieving data from the database and building/updating the website
- Database
    - Single source of truth that keeps all data
-Einspruchsgesetz Collector
    - collects data from the internet and updates the database with it

Each part serves a unique purpose and communicates with the others via a http api.
The only callable end points are provided by the database part of the project, the others merely call it.

## Webserver
- first idea: use ssg to template the site, with the webserver generating .md files and regenerating
the site as well as deploy it
## Collector
### General Remarks
- collects data from the internet via 
    - api calls
    - web scraping
- updates the database regularly about new files
- receives updated ressource locators from the database
- parses data for the db

## Database
### General Remarks
- we keep the possibility open to use file locators and save all files we downloaded
- is our single source of truth
- has a reading and writing api via http (write is verified)
- supports following collector api processes
    1. Updates
        1. receive update call
        2. return locator updates to the collector
    2. Hashed Content Update
        1. receive list of hashes of new ressources
        2. compare internally and return unknown hashes
        3. receive ressources from the collector
- supports reading api processes in some way
- ensures data is well-formed
- rate limits the calls to avoid congestion/ddos

### Data in the Database
- Metainformation
    - list of hashes and associated collector's sources
    - list of associated data file's locations, nullable (meaning no file present)
- Gesetzesvorgang
    - Parlament
    - Name des Gesetzesvorhabens
    - Kurzbeschreibung
    - Zustand
    - weitere Informationen

- Tagesordnungen

#### Gesetzesvorgänge und ihre Tabellen
Folgende Tabellen für Gesetztesvorhaben in allen Bundesländern sowie dem Bundestag

Gesetzesvorhaben:
    - id
    - titel
    - off Titel
    - initiator (s. initiatoren)
    - parlament (s. Parlamente)
    - url_gesblatt
    - id_gesetzesblatt
    - federf_ausschuss (s. Ausschüsse)
    - 
    - trojaner (y/n/-)
    - verfassungsändernd (y/n)

Gesetzeseigenschaft
    - id 
    - title (verfassungsändernd, zustimmungsgesetz, einspruchgesetz, volksbegehren)

gesetzeseigenschaften_gesetze:
    - ges_id (s. Gesetzesvorhaben)
    - eig_id (s. Eigenschafts)

Schlagworte:
    - id
    - schlagwort
    - beschreibung

schlagworte_gesetze:
    - ges_id (s. Gesetzesvorhaben)
    - schlagwort_id (s. Schlagworte)

Sonstige_bezeichner:
    - gesetzes_id
    - typ
    - textfeld

Dokumente:
    - id
    - off. id (str)
    - typ (s. dokumententypen, z.B.: Drucksache, Änderungsantrag, Beschlussempfehlung)
    - url
    - file
    - hash
    - collector-url

Initiatoren:
    - id
    - name
    - organisation
    - url
    Bsp: (0/Marco Buschmann/BMJ/https://bmj.de)

status_gesetze:
    - ges_id (s. Gesetzesvorhaben)
    - status_id (s. status)
    - datum
    - id_abstimmung (s. Abstimmungen)
    - active: bool (ist das der letzte Zustand?)
    BsP:(0/1/12.04.2024/yes)

Abstimmungen:
    - id
    - typ (volksbegehren/parlamentsabst)
    - namentlich (y/n)
    - url

abst_ergebnisse:
    - fraktion (dafür/dagegen/CDU/FDP/Grünen/Regierungsfraktionen)
    - id_abstimmung
    - anteil

Status:
    - id
    - name
    - Parlament (s. Parlamente)
    Bsp: (0/Referentenentwurf/BT)
    Bsp: (1/Erster Durchgang/BR)
    Bsp: (2/Erste Lesung/BY)
    Bsp: (3/Zweite Lesung/BW)

Ausschussberatung:
    - id_gesetz
    - id_ausschuss (s. Ausschüsse)
    - datum
    - id_dokument

TOPs:
    - id
    - datum
    - parlament
    - url

top_gesetze:
    - ges_id (s. Gesetzesvorgang)
    - tops_id (s. TOPs)
    - TOP (=die tatsächliche Nummer auf den Tagesordnungspunkte)
    - titel: Text
    - id_abstimmung (s. Abstimmungen)
    - id_dokument (s.Dokumente)
