# Der Landtagszusammenfasser

## Projektübersicht

Der Landtagszusammenfasser ist ein Tool, das Informationen und Zusammenfassungen von Landesgesetzen in Deutschland automatisiert sammelt, 
verarbeitet und präsentiert. Es soll dabei helfen, politische Prozesse transparenter und für Bürger zugänglicher zu machen.
Inspiriert ist das ganze vom [Bundestagszusammenfasser](https://bundestagszusammenfasser.de) von Sabrina Gehder.

## Notizen

- Vorgaben in Sachen Pflichtinfos die in einem Gesetzesentwurf drinnen stehen müssen unterscheiden sich eventuell zwischen Bund und Länder
- Referentenentwürde sind oft (z.B. in Niedersachsen) nicht öffentlich, sondern erst sobald sie im Parlament sind
- Gesetzesvorgänge sind ähnlich, aber nicht dieselben
- Dokumente auf der Website des Bayrischen Landtages haben manchmal eine Info dazu wo die Entwürfe gerade hängen
- Vorbereitungsphase der Gesetze wird manuell gehandelt. Sie hatte überlegt scraper zu schreiben, aber meinte es hat sich bei ihr nicht gelohnt weil signal to noise ratio viel zu hoch und oft nicht klar ist wie alt etwas ist
- Sobald es im Bundesrat ist, ruft sie die Infos über die API des Bundestages/Bundesrates ab. Dadurch kann selbst der Text direkt abgerufen werden. Der Text geht dann zu GPT, was dann eine Zusammenfassung schreibt. Dadruch kriegt man auch mit, ob sich der Beratungsstand ändert
- Seit neuestem gibt es auch eine API die einem die Tagesordnung gibt
- ChatGPT liest auch Protokolle und extrahiert Drucksachen
- Durch tägliche Statusabfrage Infos über Änderungen
- ca. 10-30 Gesetzesbesprechungen pro Sitzungswoche. 400 Vorgänge nach 3 Jahren von der Regierung, nochmal 150 von der Opposition
- Sabrina speichert nur die Zusammenfassungen, nicht die Gesetzestexte selbst.
- Fürchtet, dass der Vorbereitungspart der Gesetzesentwürfe nicht öffentlich in den Ländern ist
- LLMs lokal laufen lassen
- Texte sind sehr lang, token window muss also passen. Anthropic (?) LLM hat sie schon mal versucht, aber das hat nicht gut funktioniert.
- Für 5% der Texte reicht nicht mal das Token window von GPT4, weil sei 300 Seiten lang sind 
- Von ihrer Seite aus kann alles frei sein. Sie sagt, Namen nennen ist gut, Hauptsache es exisitiert

## Technische Details
Mal sehen

## Lizenz

Copyleft