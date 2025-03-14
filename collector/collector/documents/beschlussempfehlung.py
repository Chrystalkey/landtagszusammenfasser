from documents import Document
from oapicode.openapi_client import models

class DSEntwurf(Document):
    drucksnr = None
    meinung = None

    def __init__(self, session, testing_mode: bool = False):
        super.__init__(self, models.Doktyp.BESCHLUSSEMPF, session, testing_mode)
        self.drucksnr = None
        self.meinung = None

    def semantic_extraction(self):
        prompt = """Extrahiere folgende Daten aus dem nachfolgenden Text:
        drucksachennummer,meinungsbild als Integer zwischen 1(ablehnung)über 5(zustimmung in geänderter Fassung) bis 10(zustimmung), autoren(personen),autoren(fraktionen),schlagworte, zusammenfassung der empfohlenen Änderungen falls Anwendbar, Trojanergefahr als Integer zwischen 1(keine) und 10(sicher)
        Nutze dazu eine json Data structure und weiche unter keinen Umständen davon ab. Die Outline der Datenstruktur ist: 
        {drucksnr:\"\", meinung: 0, autorpersonen: [], autoren: [], schlagworte: [], sum_changes: \"\",, trojanergf: 0}
        ENDE DES PROMPTS TEXT FOLGT"""