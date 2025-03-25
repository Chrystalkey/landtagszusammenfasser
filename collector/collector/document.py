import hashlib
import json
import os
import openapi_client.models as models
import uuid
import datetime
import logging
import pypdf
import csv
from typing import List, Optional, Dict, Any, Union
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
    @classmethod
    def testinit(cls):
        instance = cls()
        instance.link = "https://www.example.com"
        instance.title = "Testtitel"
        instance.last_mod = datetime.datetime.fromisoformat("1940-01-01T00:00:00+00:00")
        instance.full_text = ["test"]
        instance.typ = "entwurf"
        instance.hash = "testhash"
        return instance
        
class Document:
    testing_mode = False
    def __init__(self, session, url, typehint: str, config):
        self.config = config
        if config.testing_mode:
            self.testing_mode = True
            self.set_testing_values()
            return
        self.testing_mode = False
        self.session = session
        self.url = url
        self.typehint = typehint

        self.meta = DocumentMeta()
        self.autoren: Optional[List[str]] = None
        self.zusammenfassung: Optional[str] = None
        self.schlagworte: Optional[List[str]] = None
        self.trojanergefahr: int = 0 # only relevant for drucksachen
        self.meinung: Optional[int] = None # only relevant for stellungnahmen
        self.drucksnr : Optional[str] = None

        self.fileid = str(uuid.uuid4())
        self.download_success = False
        self.extraction_success = False

    def set_testing_values(self):
        self.meta = DocumentMeta.testinit()
        self.autoren = []
        self.schlagworte = ["test"]
        self.trojanergefahr = 0
        self.texte = ["test"]
        self.zusammenfassung = "test"
        self.meinung = 1
        self.download_success = True
        self.extraction_success = True
        self.fileid = str(uuid.UUID("00000000-0000-0000-0000-000000000000"))
        self.url = "https://www.example.com"
        self.typehint = "entwurf"
        self.drucksnr = "example"
        
        
    def to_json(self) -> str:
        return json.dumps(self.to_dict())
    
    @classmethod
    def from_json(cls, json_str: str):
        inst = cls.from_dict(json.loads(json_str))
        inst.testing_mode = False
        inst.fileid = None

    def __del__(self):
        self._cleanup_tempfiles()
            
    def _cleanup_tempfiles(self):
        """Clean up any temporary files created during document processing"""
        if self.fileid and os.path.exists(f"{self.fileid}.pdf"):
            try:
                os.remove(f"{self.fileid}.pdf")
            except Exception as e:
                logger.warning(f"Failed to remove temporary PDF file: {e}")
    
    @classmethod
    def from_dict(cls, dic):
        instance = cls(None, dic["url"], dic["typehint"], None)  # Create new instance
        instance.meta = DocumentMeta.from_dict(dic["meta"])
        instance.testing_mode = dic.get("testing_mode", False)
        autoren = dic.get("autoren")
        if autoren:
            instance.autoren = []
            for aut in autoren:
                instance.autoren.append(models.Autor.from_dict(aut))
        instance.drucksnr = dic.get("drucksnr")
        instance.schlagworte = dic.get("schlagworte")
        instance.trojanergefahr = dic.get("trojanergefahr", 0)
        instance.zusammenfassung = dic.get("zusammenfassung")
        instance.meinung = dic.get("meinung")
        instance.download_success = True
        instance.extraction_success = True
        return instance

    def to_dict(self):
        autoren = []
        if self.autoren:
            for aut in self.autoren:
                autoren.append(aut.to_dict())
        return {
            "meta": self.meta.to_dict(),
            "url": self.url,
            "typehint": self.typehint+"",
            "autoren": autoren,
            "typehint": self.typehint,
            "schlagworte": self.schlagworte,
            "trojanergefahr": self.trojanergefahr,
            "drucksnr": self.drucksnr,
            "zusammenfassung": self.zusammenfassung,
            "meinung": self.meinung
        }
    
    async def run_extraction(self):
        """Main method to download and extract information from a document"""
        if self.testing_mode:
            return True
        try:
            await self.download()
            self.download_success = True
        except Exception as e:
            logger.error(f"Failed to download document {self.url}: {e}")
            self._cleanup_tempfiles()
            return False
            
        try:
            await self.extract_metadata()
            await self.extract_semantics()
            self.extraction_success = True
            return True
        except Exception as e:
            logger.error(f"Failed to extract from document {self.url}: {e}")
            self._cleanup_tempfiles()
            return False

    async def download(self):
        """Download the document from the URL"""
        if self.testing_mode:
            return True
        logger.info(f"Downloading document from {self.url}")
        try:
            async with self.session.get(self.url) as response:
                if response.status != 200:
                    raise Exception(f"Failed to download document, status: {response.status}")
                
                with open(f"{self.fileid}.pdf", "wb") as f:
                    f.write(await response.read())
                    
            if not os.path.exists(f"{self.fileid}.pdf") or os.path.getsize(f"{self.fileid}.pdf") == 0:
                raise Exception("Downloaded file is empty or doesn't exist")
        except Exception as e:
            logger.error(f"Download error for {self.url}: {e}")
            raise
    
    async def extract_metadata(self) -> DocumentMeta:
        """Extract metadata from the PDF file"""
        if self.testing_mode:
            return True
        logger.debug(f"Extracting PDF Metadata for Url {self.url}, using file {self.fileid}.pdf")
        
        try:
            with open(f"{self.fileid}.pdf", "rb") as f:
                reader = pypdf.PdfReader(f)
                
                # Extract metadata from PDF
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
                
                # Calculate file hash for document identification
                f.seek(0)
                doc_hash = hashlib.file_digest(f, "sha256").hexdigest()
                
                # Extract text from all pages
                full_text = []
                for page in reader.pages:
                    extracted_text = page.extract_text()
                    if extracted_text:
                        full_text.append(extracted_text)
                    
                # Check if we got any text from the document
                if not full_text:
                    logger.warning(f"No text extracted from PDF: {self.url}")
                
                # Create metadata object
                self.meta = DocumentMeta.from_dict({
                    "title": meta.title if hasattr(meta, 'title') and meta.title else None,
                    "link": self.url,
                    "hash": doc_hash,
                    "typ": self.typehint+"",
                    "last_mod": dtime.astimezone(datetime.timezone.utc).isoformat(),
                    "full_text": full_text
                })
                
        except Exception as e:
            logger.error(f"Error extracting metadata from PDF: {e}")
            raise
        finally:
            self._cleanup_tempfiles()

    async def extract_semantics(self):
        """Extract semantic information using the LLM"""
        if self.testing_mode:
            return True
        if not self.meta.full_text or all(not text for text in self.meta.full_text):
            logger.warning(f"No text to analyze in document {self.url}")
            self.meta.title = self._get_default_title()
            return
        
        # Different prompts for different document types
        if self.typehint == "entwurf":
            await self._extract_drucksache_semantics()
        elif self.typehint == "stellungnahme":
            await self._extract_stellungnahme_semantics()
        elif self.typehint == "protokoll":
            self._extract_protokoll_semantics()
        else:
            self._extract_default_semantics()
    
    def _get_default_title(self):
        """Get a default title based on document type"""
        type_titles = {
            "entwurf": "Gesetzesentwurf",
            "stellungnahme": "Stellungnahme",
            "protokoll": "Protokoll",
            "sonstig": "Dokument"
        }
        return type_titles.get(self.typehint, "Unbekanntes Dokument")
    
    async def _extract_drucksache_semantics(self):
        """Extract semantics for a 'drucksache' document"""
        prompt = """Titel;Autorengruppen wie z.B. Regierungen/Parteien/Parlamentarische/Nicht-parlamentarische Gruppen als Liste;Autoren als Liste aus Tupeln{"psn", "org"};Schlagworte als Liste;Zahl zwischen 0 und 10, die die Gefahr einschätzt dass im Gesetzestext Fachfremde Dinge untergeschoben werden sollen;Kurzzusammenfassung der Intention, dem Fokus, betroffenen Gruppen und anderen wichtigen Informationen aus dem Text in 150-250 Worten
Anführungszeichen ein. Antworte mit nichts anderem als den gefragten Informationen.
Gib die Antwort als JSON aus mit den Feldern: {"titel", "gruppen", "personen", "schlagworte", "troja", "summary"}
WEICHE UNTER KEINEN UMSTÄNDEN VON DER JSON-STRUKTUR AB
ENDE DES PROMPTS"""
        
        try:
            full_text = " ".join(self.meta.full_text).strip()
            if len(full_text) <= 20:
                logger.warning(f"Extremely short text: `{full_text}` within a document. This might hint at a non-machine readable document. The URL ist `{self.url}`")
                
            response = await self.config.llm_connector.generate(prompt, full_text)
            
            # Parse the response, handle potential edge cases
            object = None
            try:
                object = json.loads(response[7:-3])
            except Exception as e:
                logger.warning(f"Invalid response format from LLM: {response}")
                self._set_default_values()
                return
            autoren = []
            for ap in object["personen"]:
                autoren.append(models.Autor.from_dict({
                    "person": ap["psn"],
                    "organisation": ap["org"]
                }))
            for ao in object["gruppen"]:
                autoren.append(models.Autor.from_dict({
                    "organisation": ao
                }))
            self.meta.title = object["titel"]
            self.autoren = autoren
            self.schlagworte = object["schlagworte"]
            self.trojanergefahr = object["troja"]
            self.zusammenfassung = object["summary"]
                
        except Exception as e:
            logger.error(f"Error extracting drucksache semantics: {e}")
            self._set_default_values(self.typehint)
    
    async def _extract_stellungnahme_semantics(self):
        """Extract semantics for a 'stellungnahme' document"""
        prompt = """Titel;Autorengruppen wie z.B. Regierungen/Parteien/Parlamentarische/Nicht-parlamentarische Gruppen als Liste;Autoren als Liste aus Objekten{"psn", "org"};Schlagworte als Liste;Zahl zwischen 0 und 5, die ein Meinungsbild angibt;Kurzzusammenfassung Stellungnahme, der Meinung und Kritik, betroffenen Gruppen und anderen wichtigen Informationen aus dem Text in 150-250 Worten
Anführungszeichen ein. Antworte mit nichts anderem als den gefragten Informationen.
Gib die Antwort als JSON aus mit den Feldern: {"titel": "", "gruppen" : [], "personen": [{"psn": "", "org": ""}], "schlagworte": [], "meinung": <int>, "summary": ""}
WEICHE UNTER KEINEN UMSTÄNDEN VON DER JSON-STRUKTUR AB
ENDE DES PROMPTS
"""
        try:
            full_text = " ".join(self.meta.full_text).strip()
            if len(full_text) <= 20:
                logger.warning(f"Extremely short text in stellungnahme: `{full_text}`. URL: `{self.url}`")
                
            response = await self.config.llm_connector.generate(prompt, full_text)
            
            # Parse the response, handle potential issues
            object = None
            try:
                object = json.loads(response[7:-3])
            except Exception as e:
                logger.warning(f"Invalid response format from LLM: {response}")
                self._set_default_values("stellungnahme")
                return
            self.meta.title = object["titel"]
            self.schlagworte = object["schlagworte"]
            self.meinung = object["meinung"]
            autoren = []
            for ap in object["personen"]:
                autoren.append(models.Autor.from_dict({
                    "person": ap["psn"],
                    "organisation": ap["org"]
                }))
            for ao in object["gruppen"]:
                autoren.append(models.Autor.from_dict({
                    "organisation": ao
                }))
            self.autoren = autoren

        except Exception as e:
            logger.error(f"Error extracting stellungnahme semantics: {e}")
            logger.error(f"Output of LLM:\n{response}")
            self._set_default_values("stellungnahme")
    
    def _extract_protokoll_semantics(self):
        """Set default values for a protokoll document"""
        self.meta.title = "Protokoll"
        self.autoren = None
        self.autorpersonen = None
        self.schlagworte = None
        self.trojanergefahr = 0
        self.texte = []
        self.zusammenfassung = None
    
    def _extract_default_semantics(self):
        """Set default values for an unknown document type"""
        self.meta.title = f"Dokument ("+self.typehint+")"
        self.autoren = None
        self.autorpersonen = None
        self.schlagworte = None
        self.trojanergefahr = 0
        self.texte = []
        self.zusammenfassung = None
    
    def _set_default_values(self, doc_type=None):
        """Set default values for a document when extraction fails"""
        if not doc_type:
            doc_type = self.typehint + ""
            
        defaults = {
            "entwurf": {
                "title": "Drucksache ohne Titel",
                "trojanergefahr": 0,
                "texte": []
            },
            "stellungnahme": {
                "title": "Stellungnahme",
                "meinung": 0
            },
            "protokoll": {
                "title": "Protokoll"
            },
            "default": {
                "title": f"Dokument ("+self.typehint+")"
            }
        }
        
        # Get defaults for this document type or use generic defaults
        type_defaults = defaults.get(doc_type, defaults["default"])
        
        # Set the title
        self.meta.title = type_defaults.get("title")
        
        # Set other defaults
        if doc_type == "entwurf":
            self.trojanergefahr = type_defaults.get("trojanergefahr", 0)
            self.texte = type_defaults.get("texte", [])
        elif doc_type == "stellungnahme":
            self.meinung = type_defaults.get("meinung", 0)

    def package(self) -> models.Dokument:
        """Package the document information for the API"""
        # Ensure all required fields are present
        return models.Dokument.from_dict({
            "titel": self.meta.title or "Ohne Titel",
            "drucksnr" : self.drucksnr,
            "volltext": " ".join(self.meta.full_text).strip() if self.meta.full_text else "",
            "autoren": self.autoren if self.autoren else [],
            "schlagworte": deduplicate(self.schlagworte if self.schlagworte else []),
            "hash": self.meta.hash,
            "zp_modifiziert": self.meta.last_mod,
            "zp_referenz": self.meta.last_mod,
            "link": self.url,
            "typ": self.typehint+"",
            "zusammenfassung": self.zusammenfassung.strip() if self.zusammenfassung else None
        })

def deduplicate(ls: list) -> list:
    x = set(ls)
    return list(x)