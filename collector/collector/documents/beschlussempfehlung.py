from documents import Document
from oapicode.openapi_client import models
from collector.config import Configuration

class DSEntwurf(Document):
    drucksnr = None
    meinung = None

    def __init__(self, session, config: Configuration):
        self.drucksnr = None
        self.meinung = None
        super.__init__(self, models.Doktyp.BESCHLUSSEMPF, session, config)

    def semantic_extraction(self):
        prompt = """Extrahiere folgende Daten aus dem nachfolgenden Text:
        drucksachennummer,meinungsbild als Integer zwischen 1(ablehnung)über 5(zustimmung in geänderter Fassung) bis 10(zustimmung), autoren(personen),autoren(fraktionen),schlagworte, zusammenfassung der empfohlenen Änderungen falls Anwendbar, Trojanergefahr als Integer zwischen 1(keine) und 10(sicher)
        Nutze dazu eine json Data structure und weiche unter keinen Umständen davon ab. Die Outline der Datenstruktur ist: 
        {drucksnr:\"\", meinung: 0, autorpersonen: [], autoren: [], schlagworte: [], sum_changes: \"\", trojanergf: 0}
        ENDE DES PROMPTS TEXT FOLGT"""