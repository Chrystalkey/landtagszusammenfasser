# Landtagszusammenfasser

## Überblick

Dieses Projekt ist ein Tool um einen Überblick über die Entwicklung von Gesetzen in den Ländern zu geben.
Es ist in drei Teile geteilt: 
1. Die Datenbankanwendung, die zentral die Daten verwaltet und die API bereitstellt
2. eine Website, die die Daten herunterlädt und Menschenlesbar aufbereitet
3. eine Sammlung and Scrapern("Collectors"), die die Daten von verschiedenen Quellen sammeln und in die Datenbank einspeichern

Die Lese-API ist dabei ohne Authentifizierung verfügbar. Die Schreib-API (also die Collectors) sind dagegen mit einer API-Key Authentifizierung versehen um
Spam und Missbrauch zu verhindern.

## API Kurzbeschreibung

Die API ist in vier Teile geteilt:
- GET /api/v1/gesetzesvorhaben oder GET /api/v1/gesetzesvorhaben/{gsvh_id}
  Öffentliche Schnittstelle um Gesetzesvorhaben aus der Datenbank abzurufen, ohne authentifizierung
- POST /api/v1/gesetzesvorhaben
  Schnittstelle der Collectors, die neue Gesetzesvorhaben in die Datenbank einfügen ohne den internen Zustand der Datenbank zu kennen
- GET/DELETE /api/v1/auth
  Schnittstelle um API-Keys zu verwalten
- PUT /api/v1/gesetzesvorhaben/{gsvh_id}
  Adminschnittstelle um den exakten Stand eines Gesetzesvorhabens zu editieren

Für details über die Schnittstellen selbst siehe die [Spezifikation](./specs/openapi.yml)

### Authentifizierungskonzept
Die Authentifizierung basiert auf API-Keys. Diese werden über die Datenbank vergeben und gelöscht. Dazu ist die Schnittstelle /api/v1/auth zuständig.
API-Keys können einen von drei Scopes zugeordnet sein:

- KeyAdder: Kann neue API-Keys erstellen
- Admin: Kann alle Gesetzesvorhaben editieren
- Collector: Kann neue Gesetzesvorhaben einfügen

Wobei höhere Scopes die berechtigungen der niedrigen Scopes einschließen.

Für weitere Informationen siehe [documentation/authentication.md](documentation/authentication.md).

## Projektsetup

Ein Beispiel wie man das Projekt aufsetzt findet sich in dem [Docker Compose File](../docker-compose.yml) im Rootverzeichnis.

### Arguments for LTZF-DB
```bash
Usage: ltzusfas-db [OPTIONS] --db-url <DB_URL> --keyadder-key <KEYADDER_KEY>

Options:
      --mail-server <MAIL_SERVER>
          [env: MAIL_SERVER=smtp.web.de]
      --mail-user <MAIL_USER>
          [env: MAIL_USER=]
      --mail-password <MAIL_PASSWORD>
          [env: MAIL_PASSWORD=]
      --mail-sender <MAIL_SENDER>
          [env: MAIL_SENDER=]
      --mail-recipient <MAIL_RECIPIENT>
          [env: MAIL_RECIPIENT=]
      --host <HOST>
          [env: LTZF_HOST=127.0.0.1] [default: 0.0.0.0]
      --port <PORT>
          [env: LTZF_PORT=8080] [default: 80]
  -d, --db-url <DB_URL>
          [env: DATABASE_URL=postgres://ltzf-user:ltzf-pass@localhost/ltzf]
  -c, --config <CONFIG>
          
      --keyadder-key <KEYADDER_KEY>
          The API Key that is used to add new Keys. This is not saved in the database. [env: LTZF_KEYADDER_KEY=]
      --merge-title-similarity <MERGE_TITLE_SIMILARITY>
          [env: MERGE_TITLE_SIMILARITY=] [default: 0.8]
  -h, --help
          Print help
  -V, --version
          Print version
```
### Arguments for LTZF-Collector

This one is configured via environment variables:
| Name             | Description                                  | Example      | Default or REQUIRED |
| ---- |---- |---- | ---- |
| LTZF_DATABASE    | The Host of the LTZF-DB service running      | localhost:80 | localhost:80 |
| API_KEY          | The API Key to be used for auth with LTZF-DB | abc123       | REQUIRED     |
| REDIS_HOST       | The Host of Redis Cache                      | localhost    | localhost    |
| REDIS_PORT       | The Port of Redis Cache                      | 6379         | 6379         |
| TROJAN_THRESHOLD | The Threshold (0-10) at which a station is classified as trojan | 5 | 5    |
| API_OBJ_LOG      | Directory to dump all api objects. useful for debug. unused if empty. |    |
| OPENAI_API_KEY   | OpenAI API Key for LLM related tasks.        | laskdjfoaisd | REQUIRED     |


### Arguments for the webserver
| Name        | Description                       | Example   | Default or REQUIRED |
| ---- |----  |---- | ---- |
| LTZFDB_HOST | host of the LTZF DB               | localhost | REQUIRED  |
| LTZFDB_PORT | port of the LTZF DB               | 80        | 80        |
| PORT        | HTTP Port this server listenes on | 80        | 80        |