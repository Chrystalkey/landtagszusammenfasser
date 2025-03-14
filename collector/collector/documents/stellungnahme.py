from documents import Document
from oapicode.openapi_client import models

class DSEntwurf(Document):
    organisation = None
    meinung = None

    def __init__(self, session, testing_mode: bool = False):
        super.__init__(self, models.Doktyp.STELLUNGNAHME, session, testing_mode)

    def semantic_extraction(self):
        prompt = """Extrahiere folgende Daten aus dem nachfolgenden Text: 
        Autoren, Organisation, Meinung(als Integer zwischen 1-sehr kritisch und 10-sehr positiv), passende Schlagworte, Zusammenfassung der Stellungnahme in 50-250 Worten
        Nutze dazu eine json Data structure und weiche unter keinen Umständen davon ab. Die Outline der Datenstruktur ist: 
        {autoren: [] ,organisation: \"\",meinung: 0,schlagworte: [],zusammenfassung: \"\"}
        ENDE DES PROMPTS TEXT FOLGT"""
        