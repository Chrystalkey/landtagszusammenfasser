# Der Landtagszusammenfasser

## Projektübersicht

Der Landtagszusammenfasser ist ein Tool, das Informationen und Zusammenfassungen von Landesgesetzen in Deutschland automatisiert sammelt, 
verarbeitet und präsentiert. Es soll dabei helfen, politische Prozesse transparenter und für Bürger zugänglicher zu machen.
Inspiriert ist das ganze vom [Bundestagszusammenfasser](https://bundestagszusammenfasser.de) von Sabrina Gehder.

Es ist in drei Teile geteilt: 
1. Die Datenbankanwendung, die zentral die Daten verwaltet und die API bereitstellt. Davon gibt es _genau_ eine.
2. eine Website, die die Daten herunterlädt und Menschenlesbar aufbereitet. Davon kann es theoretisch tausende geben.
3. eine Sammlung and Scrapern("Collectors"), die die Daten von verschiedenen Quellen sammeln und in die Datenbank einspeichern. Die arbeiten unabhängig von der DB, und theoretisch kann es für jede deutsche Ministeriumsseite einen eigenen Collector geben, es macht aber Sinn sie zu bündeln.

Die Lese-API ist dabei ohne Authentifizierung verfügbar. Die Schreib-API (also die Collectors) sind dagegen mit einer API-Key Authentifizierung versehen um Spam und Missbrauch zu verhindern.

Ein Beispielaufbau könnte sein:

|-Scraper für sächsisches Wirtschaftsministerium  |-Scraper für Bayerischen Landtag  
|-Scraper für sächischen Landtag                  |-Scraper für bayerisches Justizministerium  
|-Scraper für sachen-anhaltinischen Landtag       |  
|                                                 |  
Collector 1, auf Maschine 1 in Sachsen            Collector 2, auf Maschine 2 in Bayern  
                    |                                              |  
                    Datenbankanwendung------------------------------  
                    Auf Maschine 3 bei Hetzner  
                    |  
                    |-------------------------------|  
Website die alle Bundesvorgänge zusammmenfasst      Website die alle Bayerischen Vorgänge anzeigt ...  


## Dokumentation
Die Grundlegenden Konzepte, API-Beschreibung und Projektsetup ist in [docs/README.md](docs/README.md). Geh und lies, junger Padawan!

## Contributing
Eine grundlegende Setup-Beschreibung für die Projekte ist in [SETUP.md](SETUP.md), lies aber bitte auch [docs/README.md](docs/README.md).
Für einen schönen Überblick zu was genau du deine Aufmerksamkeit wenden könntest, konsultiere doch [CONTRIBUTING.md](CONTRIBUTING.md), lies die Feature wishlist in den issues durch oder allgemein die issues.

## Lizenz

Do whatever you want, just please mention our names and/or link to our work :)