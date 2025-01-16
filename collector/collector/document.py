import hashlib
import json
import os
import openapi_client.models as models
import uuid
import datetime
import logging
import pypdf
import csv
from .llm_connector import LLMConnector

logger = logging.getLogger(__name__)

class DocumentMeta:
    def __init__(self):
        self.link = None
        self.title = None
        self.last_mod = None
        self.full_text = None
        self.hash = None
        self.typ = None
    
    @classmethod
    def from_dict(cls, dic):
        instance = cls()
        instance.link = dic["link"]
        instance.title = dic["title"]
        instance.last_mod = datetime.datetime.fromisoformat(dic["last_mod"])
        instance.full_text = dic["full_text"]
        instance.hash = dic["hash"]
        instance.typ = dic["typ"]
        return instance
    
    def to_dict(self):
        return {
            "link": self.link,
            "title": self.title,
            "last_mod": self.last_mod.astimezone(datetime.timezone.utc).isoformat(),
            "full_text": self.full_text,
            "hash": self.hash,
            "typ": self.typ
        }
        
class Document:
    def __init__(self, session, url, typehint: str, config):
        self.session = session
        self.url = url
        self.typehint = typehint

        self.meta = DocumentMeta()
        self.authoren = None
        self.autorpersonen = None
        self.zusammenfassung = None
        self.schlagworte = None
        self.trojanergefahr = 0 # only relevant for drucksachen
        self.texte = [] # only relevant for drucksachen
        self.meinung = None # only relevant for stellungnahmen

        self.config = config
        self.fileid = str(uuid.uuid4())
        
    def to_json(self):
        return json.dumps(self.to_dict())
    
    @classmethod
    def from_json(cls, json_str):
        return cls.from_dict(json.loads(json_str))

    def __del__(self):
        if os.path.exists(f"{self.fileid}.pdf"):
            os.remove(f"{self.fileid}.pdf")
            
    @classmethod
    def from_dict(cls, dic):
        instance = cls(None, dic["url"], dic["typehint"], None)  # Create new instance
        instance.meta = DocumentMeta.from_dict(dic["meta"])
        instance.authoren = dic["autoren"]
        instance.autorpersonen = dic["autorpersonen"]
        instance.schlagworte = dic.get("schlagworte")
        instance.trojanergefahr = dic.get("trojanergefahr")
        instance.texte = dic.get("texte")
        instance.zusammenfassung = dic.get("zusammenfassung")
        return instance

    def to_dict(self):
        return {
            "meta": self.meta.to_dict(),
            "url": self.url,
            "typehint": self.typehint,
            "autoren": self.authoren,
            "autorpersonen": self.autorpersonen,
            "schlagworte": self.schlagworte,
            "trojanergefahr": self.trojanergefahr,
            "texte": self.texte,
            "zusammenfassung": self.zusammenfassung
        }
    async def run_extraction(self):
        await self.download()
        await self.extract_metadata()
        await self.extract_semantics()

    async def download(self):
        async with self.session.get(self.url) as response:
            with open(f"{self.fileid}.pdf", "wb") as f:
                f.write(await response.read())
    
    async def extract_metadata(self) -> DocumentMeta:
        logger.debug(f"Extracting PDF Metadata for Url {self.url}, writing to file {self.fileid}.pdf")
        with open(f"{self.fileid}.pdf", "rb") as f:
            reader = pypdf.PdfReader(f)
            meta = reader.metadata
            dtime: datetime.datetime = datetime.datetime.now()
            try:
                preformed_date = meta.modification_date or meta.creation_date
                dtime = preformed_date or datetime.datetime.now()
            except Exception as e:
                logger.warning(
                    f"Datetime Conversion failed: {e} with DocumentInformation Class {meta}"
                )
                dtime = datetime.datetime.now()
            self.meta = self.meta.from_dict({
                "title": None,
                "link": self.url,
                "hash": hashlib.file_digest(f, "sha256").hexdigest(),
                "typ": self.typehint,
                "last_mod": dtime.astimezone(datetime.timezone.utc).isoformat(),
                "full_text": []
            })
            for page in reader.pages:
                self.meta.full_text.append(page.extract_text())

        if os.path.exists(f"{self.fileid}.pdf"):
            os.remove(f"{self.fileid}.pdf")

    async def extract_semantics(self):
        prompt_drcks = """Extrahiere folgende Metadaten aus dem Nachfolgenden Text:

Titel;Autorengruppen wie z.B. Regierungen, Parteien, Parlamentarische oder Nicht-parlamentarische Gruppen als Liste;Autoren als Personen als Liste;Schlagworte als Liste;Zahl zwischen 0 und 10, die die Gefahr einschätzt dass im Gesetzestext Fachfremde Dinge untergeschoben wurden;Betroffene Gesetzestexte als Liste;Kurzzusammenfassung der Intention, dem Fokus, betroffenen Gruppen und anderen wichtigen Informationen aus dem Text in 150-250 Worten

Nutze die CSV Struktur wie oben beschrieben, weiche nicht davon ab. Formatiere Listen mit ',' als Separator. Falls eine Information nicht vorhanden sein sollte, füge stattdessen "None" ohne Anführungszeichen ein. Antworte mit nichts anderem als den gefragten Informationen.
WEICHE UNTER KEINEN UMSTÄNDEN VON DER CSV-STRUKTUR AB
END PROMPT"""
        prompt_stellungnahme = """Extrahiere folgende Metadaten aus dem Nachfolgenden Text:

Titel;Autorengruppen wie z.B. Regierungen, Parteien, Parlamentarische oder Nicht-parlamentarische Gruppen als Liste;Autoren als Personen als Liste;Schlagworte als Liste;Meinung(-1/0/1);Kurzzusammenfassung Stellungnahme, der Meinung und Kritik, betroffenen Gruppen und anderen wichtigen Informationen aus dem Text in 150-250 Worten

Nutze die CSV Struktur wie oben beschrieben, weiche nicht davon ab. Formatiere Listen mit ',' als Separator. Falls eine Information nicht vorhanden sein sollte, füge stattdessen "None" ohne Anführungszeichen ein. Antworte mit nichts anderem als den gefragten Informationen.
WEICHE UNTER KEINEN UMSTÄNDEN VON DER CSV-STRUKTUR AB
END PROMPT
"""
        # titel,
        # autorengruppen,
        # autoren,
        # schlagworte,
        # gefahr,
        # betroffene_texte,
        # zusammenfassung
        body = ""
        try:
            # Combine all text from pages
            full_text = " ".join(self.meta.full_text)
            if len(full_text) <= 20:
                logger.warning(f"Extremely short text: `{full_text}` within a document. This might hint at a non-machine readable document. The URL ist `{self.url}`")
            response = ""
            # Get response from LLM
            if self.typehint == "drucksache":
                response = await self.config.llm_connector.generate(prompt_drcks, full_text)
                
                # Parse the response
                body = body = "\n".join(response.strip().split("\n")[1:])
                reader = csv.reader([" ".join(body)], delimiter=";")
                parts = [part for part in reader][0]
                if len(parts) == 7:
                    self.meta.title = parts[0] if parts[0] != 'None' else "Ohne Titel"
                    self.authoren = parts[1].split(',') if parts[1] != 'None' else None
                    self.autorpersonen = parts[2].split(',') if parts[2] != 'None' else None
                    self.schlagworte = parts[3].split(',') if parts[3] != 'None' else None
                    self.trojanergefahr = int(parts[4]) if parts[4] != 'None' else 0
                    self.texte = parts[5].split(',') if parts[5] != 'None' else []
                    self.zusammenfassung = parts[6] if parts[6] != 'None' else ""
                else:
                    logger.error(f"Invalid response format from LLM: {response}")
            elif self.typehint == "stellungnahme":
                response = await self.config.llm_connector.generate(prompt_stellungnahme, full_text)
                # Parse the response
                body = response.strip().split("\n")[1:]
                reader = csv.reader([" ".join(body)], delimiter=";")
                parts = [part for part in reader][0]
                if len(parts) == 6:
                    self.meta.title = parts[0] if parts[0] != 'None' else "Stellungnahme"
                    self.authoren = parts[1].split(',') if parts[1] != 'None' else None
                    self.autorpersonen = parts[2].split(',') if parts[2] != 'None' else None
                    self.schlagworte = parts[3].split(',') if parts[3] != 'None' else None
                    self.meinung = int(parts[4]) if parts[4] != 'None' else 0
                    self.zusammenfassung = parts[5] if parts[5] != 'None' else ""
                else:
                    logger.error(f"Invalid response format from LLM: {response}")
            elif self.typehint == "protokoll":
                self.meta.title = "Protokoll"
                self.meta.authoren = None
                self.meta.autorpersonen = None
                self.meta.schlagworte = None
                self.meta.trojanergefahr = None
                self.meta.texte = []
                self.meta.zusammenfassung = None
            else:
                self.meta.title = "Sonstiges"

        except Exception as e:
            logger.error(f"Error extracting semantics: {e}")
            logger.error(f"response: {response}")

    def package(self) -> models.Dokument:
        return models.Dokument.from_dict({
            "titel": self.meta.title or "Ohne Titel",
            "autoren": self.authoren,
            "autorpersonen": self.autorpersonen,
            "schlagworte": self.schlagworte,
            "hash": self.meta.hash,
            "last_mod": self.meta.last_mod,
            "link": self.url,
            "typ": self.meta.typ,
            "zusammenfassung": self.zusammenfassung
        })
