# Structure of the Project
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
- Gesetzesvorgang
    - [meta] list of source hashes from collector sources
    - [meta] associated data files' locators
    - "Entwurf"
    - "Beratung"
    - "Beschlossen"
- Tagesordnungen
