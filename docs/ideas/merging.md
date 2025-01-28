# Zusammenführung von API-Objekten

## Struktur

Das Einfügen in die Datenbank geschieht im Allgemeinen über die 
Zusammenführung verschiedener API-Objekte.

In mehreren Schritten: 
**Objekte ohne Ähnlichkeit**
1. Collector schickt erstes Mal Entwurf an die Datenbank
2. Datenbank prüft auf Ähnlichkeit, Kein ähnliches Objekt wurde gefunden
3. Datenbank fügt das Objekt vollständig ein

**Objekte mit Ähnlichkeit**
4. Collector schickt einen Gesetzesentwurf mit einer neuen Station
5. Datenbank prüft auf Ähnlichkeit des Gesetzesentwurfs, findet einen
6. Datenbank integriert neue Station in den Gesetzesentwurf

## Ähnlichkeitskriterien
Für alle Datenbankobjekte gibt es Ähnlichkeitskriterien.
- Alle boolean values, zeitpunkte, urls, schlagworte werden ignoriert
- Alle Enumeration values und IDs müssen exakt gleich sein (e.g. typ)
- alle titel-strings müssen sich ähnlich genug sein
- weitere objektspezifische Kriterien

### Gesetzesvorhaben
Außer den allgemeinen Kriterien muss für Gesetzesvorhaben gelten:
- Alle Stationen müssen dem Gesetzgebungstrack entsprechen.
- ein gleicher Identifikator zählt wie ein gleicher Titel

### Station
- Wenn die Dokumente exakt gleich sind, wird die Station als gleich gewertet und die Stellungnahme angepasst.
- Wenn die Stellungnahme exakt gleich ist, wird die Station als gleich gewertet und die Dokumente angepasst. 

### Dokument
Hash muss gleich sein, alle anderen Kriterien sind gleichgültig

### Stellungnahme
Stellungnahmengleichheit werden nach ihrem Dokument entschieden
