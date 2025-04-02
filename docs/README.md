# Landtagszusammenfasser

## API Kurzbeschreibung

Die API ist in drei Teile geteilt:

### 1. /api/v1/vorgang[/{vorgang_id}]
- GET /api/v1/vorgang oder GET /api/v1/vorgang/{vorgang_id}
  Öffentliche Schnittstelle um Vorgänge aus der Datenbank abzurufen, ohne authentifizierung
- PUT /api/v1/vorgang
  Schnittstelle der Collectors, die neue Vorgänge in die Datenbank einfügen ohne den internen Zustand der Datenbank zu kennen
- PUT/DELETE /api/v1/vorgang/{vorgang_id}
  Adminschnittstelle um den exakten Stand eines Vorganges zu setzen oder den gesamten Vorgang zu löschen.

### 2. /api/v1/auth
- POST/DELETE /api/v1/auth
  Schnittstelle um API-Keys zu verwalten

### 3. /api/v1/sitzung[/{sid}]
- GET /api/v1/sitzung oder GET /api/v1/sitzung/{sid}
  Öffentliche Schnittstelle um Sitzungen aus der Datenbank abzurufen, ohne Authentifizierung
- PUT /api/v1/sitzung/{sid}
  Adminschnittstelle um den exakten Stand einer Sitzung zu setzen
- DELETE /api/v1/sitzung/{sid}
  Adminschnittstelle um eine Sitzung zu löschen

### 4. /api/v1/kalender[/{parlament}/{datum}]
- GET /api/v1/kalender
  Öffentliche Schnittstelle um Sitzungsdaten gefiltert abzurufen, ohne Authentifizierung
- GET /api/v1/kalender/{parlament}/{datum}
  Öffentliche Schnittstelle um Sitzungsdaten für ein bestimmtes Parlament an einem bestimmten Datum abzurufen
- PUT /api/v1/kalender/{parlament}/{datum}
  Adminschnittstelle um Sitzungsdaten für ein bestimmtes Parlament an einem bestimmten Datum zu setzen

Für details über die Schnittstellen selbst siehe die [Spezifikation](./specs/openapi.yml)

## Allgemeines Datenkonzept
Das Datenkonzept hinter diesem Dienst besteht aus zwei Hauptsäulen: Dem `Vorgang` und der `Sitzung`.

Ein Vorgang ist die Gesamtheit aller `Station`en, die ein Gesetz oder Antrag in einem oder mehreren Parlamenten durchläuft.
Eine `Station` beschreibt hierbei einen wichtigen Schritt in den Beratungen des Vorganges. 

_Zum Beispiel:_
_Ein Gesetz zur `Haarfärbeverpflichtung` wird eingebracht von der Regierung von Bayern ohne vorherige Vorhabensveröffentlichung._
_Die erste Station im Parlament ist die `Initiative`._
_Darauf folgt eine `1. Lesung` (nächste `Station`) und eine Ausschussberatung in den Aussschüssen für Inneres und Gemüseaufläufe und dem_
_Ausschuss für Farbentheorie (= Zwei Stationen)._
_Anschließend stimmt das Parlament dem Gesetz zu (=Station mit Typ `parl-akzeptanz`), und das Gesetz wird im Gesetzblatt veröffentlicht (=Station mit Typ `postparl-gsblt`)_
_Das Gesetz tritt am 7.12.2077 in Kraft (=Station mit Typ `postparl-kraft`)_


Eine Ausschussberatung besteht aus mehreren Sitzungen und Anhörungen des betreffenden Ausschusses. Da (Ausschuss-)sitzungen aber nicht nur einen 
sondern mehrere Vorgänge behandeln können, sind sie ein eigenes Datenkonzept.

### Sitzungskonzept
Eine `Sitzung` ist ein allgemeineres Konzept als eine Ausschusssitzung und kann für verschiedene Arten von Zusammentreffen in einem Parlament verwendet werden, z.B. auch für eine Plenarsitzung. Sie hat einen Termin, ein zugehöriges `Gremium` und eine Liste von `TOP`s.

Eine Sitzung kann durch das Hinzufügen von geladenen Experten zu einer `Anhörung` werden, was allerdings nur relevant für tatsächliche Ausschusssitzungen ist. 

#### Tagesordnungspunkte
Ein Tagesordnungspunkt kann mehrere Dokumente enthalten, auf die sie sich beziehen. Das können z.B. Gesetzentwürfe sein, aber auch Stellungnahmen und andere Dinge.
Es gibt eine m:n beziehung zwischen Dokument/Vorgang und Tagesordnungspunkt.
Die Assoziation mit Vorgängen wird dynamisch zum Zeitpunkt der Abfrage erstellt, d.h. es wird geschaut in 
- station.dokumente
- station.stellungnahmen
und die jeweilige Vorgangs-id ausgegeben.

### Kalenderkonzept
Der `Kalender` bietet eine zeitliche Übersicht über alle Sitzungen in einem oder mehreren Parlamenten. Er ermöglicht das Abrufen von Sitzungen nach Datum, Parlament, Gremium und anderen Filterkriterien.

### Authentifizierungskonzept
Die Authentifizierung basiert auf API-Keys. Diese werden über die Datenbank vergeben und gelöscht. Dazu ist die Schnittstelle /api/v1/auth zuständig.
API-Keys können einen von drei Scopes zugeordnet sein:

- KeyAdder: Kann neue API-Keys erstellen
- Admin: Kann alle Gesetzesvorhaben editieren
- Collector: Kann neue Gesetzesvorhaben einfügen

Wobei höhere Scopes die berechtigungen der niedrigen Scopes einschließen.

Für weitere Informationen siehe [documentation/authentication.md](documentation/authentication.md).

## Grundlegende Projektstruktur
Das Projekt besteht aus drei vollständig unabhängigen Komponenten:
1. den Collectors (/collector)
2. dem Backend (/database)
3. der Website (/webserver)

Diese drei haben klar getrennte Aufgabenbereiche und sind auf größtmögliche Crowdsourcing-Bequemlichkeit ausgelegt. 
Die drei Komponenten sind dabei zusammengebunden über eine geteilte Definition der [HTTP-API](specs/openapi.yml), die den Datenaustausch vereinheitlicht.  

### Collectors
Die Collectors sind die Hauptquelle für Daten im System. Ein collector besteht aus mehreren Scrapern, die zyklisch Daten aus diversen Quellen extrahieren.
_Beispiel: Ein Collector, besteht aus einem Scraper für die Vorgänge im Landtag, einem für die Sitzungskalender und einem dritten der Justiz- und Wirtschaftsministerium scrapt._
Die Collectors übernehmen hier die Komplexität der Daten-sanitation, aber _nicht_ die der Deduplikation. Ein Aufruf der entsprechenden Schnittstelle ist 
Idempotent, da das Backend ein Matching vornimmt, welche Vorgänge den neuen Daten exakt entsprechen und die Daten entsprechend merged.
_Beispiel: Ein Collector sorgt dafür, dass Autoren und Organisationsnamen einheitlich sind, das wird von niemand anderem gemacht. Er ist nicht dafür verantwortlich duplizierte Vorgänge zu eliminieren; dafür ist die Datenbank zuständig. Ein Collector könnte also zweimal denselben Vorgang in die Datenbank schreiben ohne sich um eine Gesamtübersicht der Vorgänge zu sorgen._

### Backend (Datenbank)
Das Backend übernimmt die Datenverwaltung und ist die zentrale Stelle, die Überblick über die Gesamtheit des Datensatzes hat. Bis auf kleinere Input-Sanitation prüft die Datenbank nicht auf Korrektheit oder Existenz bestimmter Datenpunkte in der Außenwelt.
_Beispiel: Die Datenbank prüft Inkonsistenzbedingungen wie z.B. Deduplikation einer Autorenliste von Dokumenten, sie prüft aber nicht ob der Autor korrekt geschrieben ist oder existiert._
Sie stellt außerdem sowohl die Lese- als auch die Schreibendpunkte der HTTP-API bereit, über die das Projekt koordiniert wird und verwaltet Zugangsberechtigungen per API-Key.

### Website(Webserver)
Der dritte Teil ist das Anzeigen des vorhandenen Datensatzes. Dafür ist eine dritte Komponente im System zuständig, die Daten aus der offenen API (ohne Zugangsbeschränkung) abruft und anzeigt. Dabei können verschiedene Filter zum Einsatz kommen um den Datenverkehr zu minimieren, praktisch kann der Webserver aber alle Daten akzeptieren, die aus der Datenbank kommen und muss keine Input-Sanitation betreiben.

## Programmkonfiguration für die Drei Systemteile

Ein Beispiel wie man das Projekt aufsetzt findet sich in dem [Docker Compose File](../docker-compose.yml) im Rootverzeichnis.

### Arguments for LTZF-DB
```bash
Usage: ltzf-db.exe [OPTIONS] --db-url <DB_URL> --keyadder-key <KEYADDER_KEY>

Options:
      --mail-server <MAIL_SERVER>
      --mail-user <MAIL_USER>
      --mail-password <MAIL_PASSWORD>
      --mail-sender <MAIL_SENDER>
      --mail-recipient <MAIL_RECIPIENT>
      --host <HOST>
          [default: 0.0.0.0]
      --port <PORT>
          [default: 80]
  -d, --db-url <DB_URL>
  -c, --config <CONFIG>

      --keyadder-key <KEYADDER_KEY>
          The API Key that is used to add new Keys. This is saved in the database.
      --merge-title-similarity <MERGE_TITLE_SIMILARITY>
          [default: 0.8]
  -h, --help
          Print help
  -V, --version
          Print version
```
### Arguments for LTZF-Collector

This one is configured via environment variables:
| Name             | Description                                  | Example      | Default or REQUIRED |
| ---- |---- |---- | ---- |
| LTZF_API_HOST    | The Host of the LTZF-DB service running      | localhost:80 | localhost:80 |
| LTZF_API_KEY     | The API Key to be used for auth with LTZF-DB | abc123       | REQUIRED     |
| REDIS_HOST       | The Host of Redis Cache                      | localhost    | localhost    |
| REDIS_PORT       | The Port of Redis Cache                      | 6379         | 6379         |
| API_OBJ_LOG      | Directory to dump all api objects. useful for debug. unused if empty. |    |
| OPENAI_API_KEY   | OpenAI API Key for LLM related tasks.        | laskdjfoaisd | REQUIRED     |


### Arguments for the webserver
| Name        | Description                       | Example   | Default or REQUIRED |
| ---- |----  |---- | ---- |
| LTZF_API_HOST | host of the LTZF DB               | localhost | REQUIRED  |
| LTZF_API_PORT | port of the LTZF DB               | 80        | 80        |
| PORT          | HTTP Port this server listenes on | 80        | 80        |