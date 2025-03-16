from documents import Document
from oapicode.openapi_client import models

class DSEntwurf(Document):
    drucksnr = None
    vorwort = None
    betroffene_gesetze = None

    def __init__(self, session, testing_mode: bool = False):
        self.vorwort = None
        self.betroffene_gesetze = None
        self.drucksnr = None
        super.__init__(self, models.Doktyp.ENTWURF, session, testing_mode)

    def semantic_extraction(self):
        prompt = """Extrahiere folgende Daten aus dem nachfolgenden Text:
        drucksachennummer,autoren(personen),autoren(institutionen),schlagworte,Zusammenfassung der Intention, Zusammenfassung der Kosten, Zusammenfassung der Änderungen, Zusammenfassung der Alternativen, Trojanergefahr als Integer zwischen 1(keine) und 10(sicher)
        Nutze dazu eine json Data structure und weiche unter keinen Umständen davon ab. Die Outline der Datenstruktur ist: 
        {drucksnr:\"\", autorpersonen: [], autoren: [], schlagworte: [], sum_intent: \"\", sum_cost: \"\", sum_changes: \"\", sum_alternatives: \"\", trojanergf: 0}
        ENDE DES PROMPTS TEXT FOLGT
        """
        pass