# Project Ideas
## Structure
The project consists of three parts:
- Webserver
    - Retrieving data from the database and building/updating the website
- Database
    - Single source of truth that keeps all data
- Collector
    - collects data from the internet and updates the database with it

Each part serves a unique purpose and communicates with the others via a http api.
The only callable end points are provided by the database part of the project, the others merely call it.

## Webserver
## Collector
### General Remarks
- collects data from the internet via 
    - api calls
    - web scraping
- updates the database regularly about new files
- receives updated ressource locators from the database

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
