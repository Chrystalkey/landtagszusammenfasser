from documents import Document
from oapicode.openapi_client import models

class Redepunkt:
    redner = None
    organisation = None
    redezeit = None
    zsmf = None
    def __init__(self):
        self.redner = None
        self.organisation = None
        self.redezeit = None
        self.zsmf = None

class DSEntwurf(Document):
    redner = []

    def __init__(self, session, testing_mode: bool = False):
        self.redner = []
        super.__init__(self, models.Doktyp.PLENAR_MINUS_PROTOKOLL, session, testing_mode)
    

    def semantic_extraction(self):
        prompt = """Extrahiere folgende Daten aus dem nachfolgenden Text:
        Redner,schlagworte,zusammenfassung der Diskussion in 50-250 Worten
        Nutze dazu eine json Data structure und weiche unter keinen Umständen davon ab. Die Outline der Datenstruktur ist: 
        {redner: [], schlagworte: [], zusammenfassung: \"\"}
        ENDE DES PROMPTS TEXT FOLGT"""