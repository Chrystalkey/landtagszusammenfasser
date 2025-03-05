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
        
class Document:
    def __init__(self, session, url, typehint: str, config):
        self.session = session
        self.url = url
        self.typehint = typehint

        self.meta = DocumentMeta()
        self.authoren: Optional[List[str]] = None
        self.autorpersonen: Optional[List[str]] = None
        self.zusammenfassung: Optional[str] = None
        self.schlagworte: Optional[List[str]] = None
        self.trojanergefahr: int = 0 # only relevant for drucksachen
        self.texte: List[str] = [] # only relevant for drucksachen
        self.meinung: Optional[int] = None # only relevant for stellungnahmen
        self.drucksnr : Optional[str] = None

        self.config = config
        self.fileid = str(uuid.uuid4())
        self.download_success = False
        self.extraction_success = False
        
    def to_json(self):
        return json.dumps(self.to_dict())
    
    @classmethod
    def from_json(cls, json_str):
        return cls.from_dict(json.loads(json_str))

    def __del__(self):
        self._cleanup_tempfiles()
            
    def _cleanup_tempfiles(self):
        """Clean up any temporary files created during document processing"""
        if os.path.exists(f"{self.fileid}.pdf"):
            try:
                os.remove(f"{self.fileid}.pdf")
            except Exception as e:
                logger.warning(f"Failed to remove temporary PDF file: {e}")
    
    @classmethod
    def from_dict(cls, dic):
        instance = cls(None, dic["url"], dic["typehint"], None)  # Create new instance
        instance.meta = DocumentMeta.from_dict(dic["meta"])
        instance.authoren = dic["autoren"]
        instance.autorpersonen = dic["autorpersonen"]
        instance.schlagworte = dic.get("schlagworte")
        instance.trojanergefahr = dic.get("trojanergefahr", 0)
        instance.texte = dic.get("texte", [])
        instance.zusammenfassung = dic.get("zusammenfassung")
        instance.meinung = dic.get("meinung")
        instance.download_success = True
        instance.extraction_success = True
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
            "zusammenfassung": self.zusammenfassung,
            "meinung": self.meinung
        }
    
    async def run_extraction(self):
        """Main method to download and extract information from a document"""
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
                    "typ": self.typehint,
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
        if not self.meta.full_text or all(not text for text in self.meta.full_text):
            logger.warning(f"No text to analyze in document {self.url}")
            self.meta.title = self._get_default_title()
            return
        
        # Different prompts for different document types
        if self.typehint == "drucksache":
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
            "drucksache": "Drucksache",
            "stellungnahme": "Stellungnahme",
            "protokoll": "Protokoll",
            "sonstig": "Dokument"
        }
        return type_titles.get(self.typehint, "Unbekanntes Dokument")
    
    async def _extract_drucksache_semantics(self):
        """Extract semantics for a 'drucksache' document"""
        prompt = """Extrahiere folgende Metadaten aus dem Nachfolgenden Text:

Titel;Autorengruppen wie z.B. Regierungen/Parteien/Parlamentarische/Nicht-parlamentarische Gruppen als Liste;Autoren als Personen als Liste;Schlagworte als Liste;Zahl zwischen 0 und 10, die die Gefahr einschätzt dass im Gesetzestext Fachfremde Dinge untergeschoben wurden;Betroffene Gesetzestexte als Liste;Kurzzusammenfassung der Intention, dem Fokus, betroffenen Gruppen und anderen wichtigen Informationen aus dem Text in 150-250 Worten

Nutze die CSV Struktur wie oben beschrieben, weiche nicht davon ab. Formatiere Listen mit ',' als Separator. Falls eine Information nicht vorhanden sein sollte, füge stattdessen "None" ohne Anführungszeichen ein. Antworte mit nichts anderem als den gefragten Informationen.
WEICHE UNTER KEINEN UMSTÄNDEN VON DER CSV-STRUKTUR AB
END PROMPT"""
        
        try:
            full_text = " ".join(self.meta.full_text).strip()
            if len(full_text) <= 20:
                logger.warning(f"Extremely short text: `{full_text}` within a document. This might hint at a non-machine readable document. The URL ist `{self.url}`")
                
            response = await self.config.llm_connector.generate(prompt, full_text)
            
            # Parse the response, handle potential edge cases
            lines = response.strip().split("\n")
            if len(lines) < 2:
                logger.warning(f"Unexpected response format from LLM: {response}")
                self._set_default_values()
                return
                
            body = lines[1]  # Skip the header line
            reader = csv.reader([body], delimiter=";")
            parts = next(reader, [])
            schlagworte = parts[3].lower().split(',') if parts[3] != 'None' else None
            for sw in schlagworte:
                sw = sw.strip()
            aupersonen = parts[2].split(',') if parts[2] != 'None' else None
            for ap in aupersonen:
                ap = ap.strip()
            if len(parts) == 7:
                self.meta.title = parts[0].strip() if parts[0] != 'None' else "Drucksache ohne Titel"
                self.authoren = parts[1].split(',') if parts[1] != 'None' else None
                self.autorpersonen = aupersonen
                self.schlagworte = schlagworte
                self.trojanergefahr = int(parts[4]) if parts[4].isdigit() else 0
                self.texte = parts[5].split(',') if parts[5] != 'None' else []
                self.zusammenfassung = parts[6].strip() if parts[6] != 'None' else ""
            else:
                logger.error(f"Invalid response format from LLM: {response}")
                self._set_default_values("drucksache")
                
        except Exception as e:
            logger.error(f"Error extracting drucksache semantics: {e}")
            self._set_default_values("drucksache")
    
    async def _extract_stellungnahme_semantics(self):
        """Extract semantics for a 'stellungnahme' document"""
        prompt = """Extrahiere folgende Metadaten aus dem Nachfolgenden Text:

Titel;Autorengruppen wie z.B. Regierungen/Parteien/Parlamentarische/Nicht-parlamentarische Gruppen als Liste;Autoren als Personen als Liste;Schlagworte als Liste;Meinung(-1/0/1);Kurzzusammenfassung Stellungnahme, der Meinung und Kritik, betroffenen Gruppen und anderen wichtigen Informationen aus dem Text in 150-250 Worten

Nutze die CSV Struktur wie oben beschrieben, weiche nicht davon ab. Formatiere Listen mit ',' als Separator. Falls eine Information nicht vorhanden sein sollte, füge stattdessen "None" ohne Anführungszeichen ein. Antworte mit nichts anderem als den gefragten Informationen.
WEICHE UNTER KEINEN UMSTÄNDEN VON DER CSV-STRUKTUR AB
END PROMPT"""
        
        try:
            full_text = " ".join(self.meta.full_text).strip()
            if len(full_text) <= 20:
                logger.warning(f"Extremely short text in stellungnahme: `{full_text}`. URL: `{self.url}`")
                
            response = await self.config.llm_connector.generate(prompt, full_text)
            
            # Parse the response, handle potential issues
            lines = response.strip().split("\n")
            if len(lines) < 2:
                logger.warning(f"Unexpected stellungnahme response format: {response}")
                self._set_default_values("stellungnahme")
                return
                
            body = " ".join(lines[1:])  # Skip header, combine all other lines
            reader = csv.reader([body], delimiter=";")
            parts = next(reader, [])
            
            if len(parts) == 6:
                self.meta.title = parts[0] if parts[0] != 'None' else "Stellungnahme"
                self.authoren = parts[1].split(',') if parts[1] != 'None' else None
                self.autorpersonen = parts[2].split(',') if parts[2] != 'None' else None
                self.schlagworte = parts[3].split(',') if parts[3] != 'None' else None
                
                # Parse meinung value, with validation
                try:
                    meinung_val = int(parts[4]) if parts[4] != 'None' else 0
                    # Ensure meinung is in expected range
                    if meinung_val < -1 or meinung_val > 10:
                        logger.warning(f"Invalid meinung value: {meinung_val}, setting to 0")
                        meinung_val = 0
                    self.meinung = meinung_val
                except (ValueError, TypeError):
                    logger.warning(f"Could not parse meinung value: {parts[4]}")
                    self.meinung = 0
                    
                self.zusammenfassung = parts[5] if parts[5] != 'None' else ""
            else:
                logger.error(f"Invalid stellungnahme response format: {response}")
                self._set_default_values("stellungnahme")
                
        except Exception as e:
            logger.error(f"Error extracting stellungnahme semantics: {e}")
            self._set_default_values("stellungnahme")
    
    def _extract_protokoll_semantics(self):
        """Set default values for a protokoll document"""
        self.meta.title = "Protokoll"
        self.authoren = None
        self.autorpersonen = None
        self.schlagworte = None
        self.trojanergefahr = 0
        self.texte = []
        self.zusammenfassung = None
    
    def _extract_default_semantics(self):
        """Set default values for an unknown document type"""
        self.meta.title = f"Dokument ({self.typehint})"
        self.authoren = None
        self.autorpersonen = None
        self.schlagworte = None
        self.trojanergefahr = 0
        self.texte = []
        self.zusammenfassung = None
    
    def _set_default_values(self, doc_type=None):
        """Set default values for a document when extraction fails"""
        if not doc_type:
            doc_type = self.typehint
            
        defaults = {
            "drucksache": {
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
                "title": f"Dokument ({self.typehint})"
            }
        }
        
        # Get defaults for this document type or use generic defaults
        type_defaults = defaults.get(doc_type, defaults["default"])
        
        # Set the title
        self.meta.title = type_defaults.get("title")
        
        # Set other defaults
        if doc_type == "drucksache":
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
            "autoren": self.authoren,
            "autorpersonen": self.autorpersonen,
            "schlagworte": self.schlagworte,
            "hash": self.meta.hash,
            "letzte_modifikation": self.meta.last_mod,
            "link": self.url,
            "typ": self.typehint,
            "zusammenfassung": self.zusammenfassung.strip() if self.zusammenfassung else None
        })
