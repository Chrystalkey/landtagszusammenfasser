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
  Öffentliche Schnittstelle um Gesetzesvorhaben aus der Datenbank abzurufen
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

## Projektsetup

Ein Beispiel wie man das Projekt aufsetzt findet sich in dem [Docker Compose File](../docker-compose.yml) im Rootverzeichnis.
