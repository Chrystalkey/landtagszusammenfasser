import hashlib
import json
import os
import re
import openapi_client.models as models
import uuid
import datetime
import logging
from kreuzberg import ExtractionConfig, extract_file, TesseractConfig, PSMMode
from typing import List, Optional
from .llm_connector import LLMConnector

logger = logging.getLogger(__name__)

class DocumentMeta:
    def __init__(self):
        self.link = None
        self.title = None
        self.created = None
        self.modified = None
        self.full_text = None
        self.hash = None
        self.typ = None
    
    @classmethod
    def from_dict(cls, dic):
        instance = cls()
        instance.link = dic["link"]
        instance.title = dic["title"]
        instance.created = dic["created"]
        instance.modified = dic["modified"]
        instance.full_text = dic["full_text"]
        instance.hash = dic["hash"]
        instance.typ = dic["typ"]
        return instance
    
    def to_dict(self):
        return {
            "link": self.link,
            "title": self.title,
            "created": self.created,
            "modified": self.modified,
            "full_text": self.full_text,
            "hash": self.hash,
            "typ": self.typ
        }
    @classmethod
    def testinit(cls):
        instance = cls()
        instance.link = "https://www.example.com"
        instance.title = "Testtitel"
        instance.created = "1940-01-01T00:00:00+00:00"
        instance.modified = "1940-01-01T00:00:00+00:00"
        instance.full_text = "test"
        instance.typ = "entwurf"
        instance.hash = "testhash"
        return instance
        
class Document:
    testing_mode = False
    def __init__(self, session, url, typehint: str, config):
        self.config = config
        if config and config.testing_mode:
            self.testing_mode = True
            self.zp_referenz = None
            self.fileid = str(uuid.UUID("00000000-0000-0000-0000-000000000000"))
            self.set_testing_values()
            return
        self.testing_mode = False
        self.session = session
        self.url = url
        self.typehint = typehint
        self.zp_referenz = None

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
        return cls.from_dict(json.loads(json_str))

    def __del__(self):
        self._cleanup_tempfiles()

    def _cleanup_tempfiles(self):
        """Clean up any temporary files created during document processing"""
        try:
            if self.fileid and os.path.exists(f"{self.fileid}.pdf"):
                os.remove(f"{self.fileid}.pdf")
        except Exception as e:
            logger.warning(f"Failed to remove temporary PDF file. Exception ignored: {e}")

    @classmethod
    def from_dict(cls, dic):
        instance = cls(None, dic["url"], dic["typehint"], None)  # Create new instance
        instance.meta = DocumentMeta.from_dict(dic["meta"])
        autoren = dic.get("autoren")
        if autoren:
            instance.autoren = []
            for aut in autoren:
                instance.autoren.append(models.Autor.from_dict(aut))
        instance.drucksnr = dic.get("drucksnr")
        instance.zp_referenz = dic.get("zp_referenz")
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
            "zp_referenz": self.zp_referenz,
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
            doc_hash = None
            with open(f"{self.fileid}.pdf", "rb") as f:
                # Calculate file hash for document identification
                f.seek(0)
                doc_hash = hashlib.file_digest(f, "sha256").hexdigest()
                
            # Extract text from all pages
            extract = await extract_file(f"{self.fileid}.pdf", 
                                        config=ExtractionConfig(
                                            ocr_config=TesseractConfig(
                                                language="deu", psm=PSMMode.SINGLE_BLOCK
                                            )
                                        ))
            full_text = extract.content
            created = extract.metadata.get("created_at") if extract.metadata.get("created_at") else  datetime.datetime.now().astimezone(datetime.UTC).isoformat()
            if created.startswith("D:"):
                if created[17:19] != "":
                    created = f"{created[2:6]}-{created[6:8]}-{created[8:10]}T{created[10:12]}:{created[12:14]}:{created[14:16]}+{created[17:19]}:{created[20:22]}"
                else:
                    created = f"{created[2:6]}-{created[6:8]}-{created[8:10]}T{created[10:12]}:{created[12:14]}:{created[14:16]}+00:00"
            modified = extract.metadata.get("modified_at") if extract.metadata.get("modified_at") else  datetime.datetime.now().astimezone(datetime.UTC).isoformat()
            if modified.startswith("D:"):
                if modified[17:19] != "":
                    modified = f"{modified[2:6]}-{modified[6:8]}-{modified[8:10]}T{modified[10:12]}:{modified[12:14]}:{modified[14:16]}+{modified[17:19]}:{modified[20:22]}"
                else:
                    modified = f"{modified[2:6]}-{modified[6:8]}-{modified[8:10]}T{modified[10:12]}:{modified[12:14]}:{modified[14:16]}+00:00"
            

            title = extract.metadata.get("title") or "Ohne Titel"

            # Check if we got any text from the document
            if not full_text:
                logger.warning(f"No text extracted from PDF: {self.url}")
            
        except Exception as e:
            logger.error(f"Error extracting metadata from PDF: {e}")
            raise
        finally:
            self._cleanup_tempfiles()
        # Create metadata object
        self.meta = DocumentMeta.from_dict({
            "link": self.url,
            "title": title,
            "modified": modified,
            "full_text": full_text,
            "created": created,
            "hash": doc_hash,
            "typ": self.typehint+"",
        })

    async def extract_semantics(self):
        """Extract semantic information using the LLM"""
        if self.testing_mode:
            return True
        if not self.meta.full_text:
            logger.warning(f"No text to analyze in document {self.url}")
            self.meta.title = self._get_default_title()
            return
        
        # Different prompts for different document types
        if self.typehint == "entwurf":
            await self._extract_gesetzentwurf_semantics()
        elif self.typehint == "stellungnahme":
            await self._extract_stellungnahme_semantics()
        elif self.typehint == "redeprotokoll":
            self._extract_redeprotokoll_semantics()
        else:
            self._extract_default_semantics()
    
    def _get_default_title(self):
        """Get a default title based on document type"""
        type_titles = {
            "entwurf": "Gesetzesentwurf",
            "stellungnahme": "Stellungnahme",
            "redeprotokoll": "Redeprotokoll",
            "sonstig": "Dokument"
        }
        return type_titles.get(self.typehint, "Unbekanntes Dokument")
    async def _extract_redeprotokoll_semantics(self):
        header_prompt = """Du wirst einen Auszug aus einem Dokument erhalten. Extrahiere daraus die Daten, die in folgendem JSON-Pseudo Code beschrieben werden:
        {'titel': 'Titel des Dokuments', 'kurztitel': 'Zusammenfassung des Titels in einfacher Sprache', 'date': 'Datum auf das sich das Dokument bezieht'
        'autoren': [{'person': 'name einer person', organisation: 'name der organisation der die person angehört'}], 'institutionen': ['liste von institutionen von denen das dokument stammt']}
        sollten sich einige Informationen nicht extrahieren lassen, füge einfach keinen Eintrag hinzu (autor/institution) oder füge 'Unbekannt' ein. Halluziniere unter keinen Umständen nicht vorhandene Informationen.
        Antworte mit nichts anderem als den gefragen Informationen, formatiere sie nicht gesondert.END PROMPT\n"""
        body_prompt = """Du wirst den Text eines Plenarprotokolls erhalten. Extrahiere eine Zusammenfassung der Diskussion und Schlagworte die das Besprochene beschreiben.
        Gib dein Ergebnis in JSON aus, wiefolgt: {'schlagworte': [], 'summary': '150-250 Worte'}
        Antworte mit nichts anderem als den gefragen Informationen, formatiere sie nicht gesondert.END PROMPT
        """
        try:
            full_text = self.meta.full_text.strip()
            if len(full_text) <= 20:
                logger.warning(f"Extremely short text: `{full_text}` within a document. This might hint at a non-machine readable document. The URL ist `{self.url}`")
            header_response = await self.config.llm_connector.generate(header_prompt, full_text[0:min(3000, len(full_text))])
            body_response = await self.config.llm_connector.generate(body_prompt, full_text)
            hobj = json.loads(header_response)
            bobj = json.loads(body_response)
            self.meta.title = hobj["titel"]
            self.zp_referenz = hobj["date"]
            self.autoren = []
            for psn in hobj["autoren"]:
                self.autoren.append(models.Autor.from_dict({"person": psn["person"], "organisation": psn["organisation"]}))
            for inst in hobj["institutionen"]:
                self.autoren.append(models.Autor.from_dict({"organisation": inst}))
            self.schlagworte = bobj["schlagworte"]
            self.zusammenfassung = bobj["summary"]

        except Exception as e:
            logger.error(f"Error extracting plenarprotokoll semantics: {e}")

    async def _extract_gesetzentwurf_semantics(self):
        header_prompt = """Extrahiere aus dem folgenden Auszug aus einem Gesetzentwurf folgende Eckdaten als JSON:
        {'titel': 'Offizieller Titel des Dokuments', 'kurztitel': 'zusammenfassung des titels in einfacher Sprache', 'date': 'datum auf das sich das Dokument bezieht',
         'autoren': [{'person': 'name einer person', organisation: 'name der organisation der die person angehört'}], 'institutionen': ['liste von institutionen von denen das dokument stammt']}
          Antworte mit nichts anderem als den gefragen Informationen, formatiere sie nicht gesondert. END PROMPT\n         
        """
        body_prompt = """Extrahiere aus dem gesamttext des folgenden Gesetzes eine Liste an schlagworten, die inhaltlich bedeutsam sind sowie eine Zusammenfassung in 150-250 Worten. 
        Gib außerdem eine "Trojanergefahr" an, also einen Wert zwischen 1 und 10, der angibt wie wahrscheinlich es ist, dass die vorgeschlagenen Änderungen einem anderen Zweck dienen als es den Anschein hat.
        Formatiere sie als JSON wiefolgt:
        {'schlagworte': [], summary: '150-250 Worte', 'troja': <int>}"""
        
        try:
            full_text = self.meta.full_text.strip()
            if len(full_text) <= 20:
                logger.warning(f"Extremely short text: `{full_text}` within a document. This might hint at a non-machine readable document. The URL ist `{self.url}`")
            
            hresp = await self.config.llm_connector.generate(header_prompt, full_text[0:min(3000, len(full_text))])
            bresp = await self.config.llm_connector.generate(body_prompt, full_text)
            bobj = json.loads(bresp[8:-3] if "```" in bresp else bresp)
            hobj = json.loads(hresp[8:-3] if "```" in hresp else hresp)
            autoren = []
            for ap in hobj["autoren"]:
                autoren.append(models.Autor.from_dict({
                    "person": ap["person"],
                    "organisation": ap["organisation"]
                }))
            for ao in hobj["institutionen"]:
                autoren.append(models.Autor.from_dict({
                    "organisation": ao
                }))
            self.meta.title = hobj["titel"]
            self.autoren = autoren
            self.zp_referenz = hobj["date"]
            self.schlagworte = bobj["schlagworte"]
            self.trojanergefahr = bobj["troja"]
            self.zusammenfassung = bobj["summary"]
            logger.warning(f"gesent response: {self.zusammenfassung}")
                
        except Exception as e:
            logger.error(f"Error extracting gesetzentwurf semantics: {e}")
            logger.error(f"LLM Response: {hresp}\nas well as\n{bresp}")
            self._set_default_values(self.typehint)
    
    async def _extract_stellungnahme_semantics(self):
        header_prompt = """Extrahiere aus dem folgenden Auszug aus einem Gesetzentwurf folgende Eckdaten als JSON:
        {'titel': 'Offizieller Titel des Dokuments', 'kurztitel': 'zusammenfassung des titels in einfacher Sprache', 'date': 'datum auf das sich das Dokument bezieht',
         'autoren': [{'person': 'name einer person', organisation: 'name der organisation der die person angehört'}], 'institutionen': ['liste von institutionen von denen das dokument stammt']}
          Antworte mit nichts anderem als den gefragen Informationen, formatiere sie nicht gesondert. END PROMPT\n         
        """
        body_prompt = """Extrahiere aus dem gesamttext des folgenden Gesetzes eine Liste an schlagworten, die inhaltlich bedeutsam sind sowie eine Zusammenfassung in 150-250 Worten. 
        Gib außerdem eine "Meinung" an als einen Wert zwischen 1(grundsätzlich ablehnend) und 5(lobend), der das Meinungsbild des Dokuments wiederspiegelt
        Formatiere sie als JSON wiefolgt:
        {'schlagworte': [], summary: '150-250 Worte', 'meinung': <int>}"""

        try:
            full_text = self.meta.full_text.strip()
            if len(full_text) <= 20:
                logger.warning(f"Extremely short text in stellungnahme: `{full_text}`. URL: `{self.url}`")
                
            hresp = await self.config.llm_connector.generate(header_prompt, full_text)
            bresp = await self.config.llm_connector.generate(body_prompt, full_text)
            try:
                hobj = json.loads(hresp)
                bobj = json.loads(bresp)
            except Exception as e:
                logger.warning(f"Invalid response format from LLM: {hresp}\nor\n{bresp}")
                self._set_default_values("stellungnahme")
                return

            self.meta.title = hobj["titel"]
            self.schlagworte = bobj["schlagworte"]
            self.meinung = bobj["meinung"]
            self.zp_referenz = hobj["referenzdate"]
            autoren = []
            for ap in hobj["autoren"]:
                autoren.append(models.Autor.from_dict({
                    "person": ap["psn"],
                    "organisation": ap["org"]
                }))
            for ao in hobj["institutionen"]:
                autoren.append(models.Autor.from_dict({
                    "organisation": ao
                }))
            self.autoren = autoren

        except Exception as e:
            logger.error(f"Error extracting stellungnahme semantics: {e}")
            logger.error(f"Output of LLM:\n{hresp}\nand\n{bresp}")
            self._set_default_values("stellungnahme")
            
    async def _extract_beschlempf_semantics(self):
        header_prompt = """Extrahiere aus dem folgenden Auszug aus einem Gesetzentwurf folgende Eckdaten als JSON:
        {'titel': 'Offizieller Titel des Dokuments', 'kurztitel': 'zusammenfassung des titels in einfacher Sprache', 'date': 'datum auf das sich das Dokument bezieht',
         'autoren': [{'person': 'name einer person', organisation: 'name der organisation der die person angehört'}], 'institutionen': ['liste von institutionen von denen das dokument stammt']}
          Antworte mit nichts anderem als den gefragen Informationen, formatiere sie nicht gesondert. END PROMPT\n         
        """
        body_prompt = """Extrahiere aus dem gesamttext des folgenden Gesetzes eine Liste an schlagworten, die inhaltlich bedeutsam sind sowie eine Zusammenfassung in 150-250 Worten. 
        Gib eine "Meinung" an als einen Wert zwischen 1(grundsätzlich ablehnend) und 5(lobend), der das Meinungsbild des Dokuments wiederspiegelt
        Gib schließlich eine "Trojanergefahr" an, also einen Wert zwischen 1 und 10, der angibt wie wahrscheinlich es ist, dass die vorgeschlagenen Änderungen einem anderen Zweck dienen als es den Anschein hat.
        Formatiere sie als JSON wiefolgt:
        {'schlagworte': [], summary: '150-250 Worte', 'meinung': <int>, 'troja': <int>}"""

        try:
            full_text = self.meta.full_text.strip()
            if len(full_text) <= 20:
                logger.warning(f"Extremely short text in stellungnahme: `{full_text}`. URL: `{self.url}`")
                
            hresp = await self.config.llm_connector.generate(header_prompt, full_text)
            bresp = await self.config.llm_connector.generate(body_prompt, full_text)
            try:
                hobj = json.loads(hresp)
                bobj = json.loads(bresp)
            except Exception as e:
                logger.warning(f"Invalid response format from LLM: {hresp}\nor\n{bresp}")
                self._set_default_values("stellungnahme")
                return

            self.meta.title = hobj["titel"]
            self.schlagworte = bobj["schlagworte"]
            self.meinung = bobj["meinung"]
            self.zp_referenz = hobj["referenzdate"]
            autoren = []
            for ap in hobj["autoren"]:
                autoren.append(models.Autor.from_dict({
                    "person": ap["psn"],
                    "organisation": ap["org"]
                }))
            for ao in hobj["institutionen"]:
                autoren.append(models.Autor.from_dict({
                    "organisation": ao
                }))
            self.autoren = autoren

        except Exception as e:
            logger.error(f"Error extracting stellungnahme semantics: {e}")
            logger.error(f"Output of LLM:\n{hresp}\nand\n{bresp}")
            self._set_default_values("stellungnahme")
    def _extract_default_semantics(self):
        """Set default values for an unknown document type"""
        self.meta.title = f"Dokument ("+self.typehint+")"
        self.autoren = None
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
            "redeprotokoll": {
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
        if self.zp_referenz:
            if re.fullmatch(r"\d{2}.\d{2}.\d{4}", self.zp_referenz):
                # reformat the date string
                rdate = self.zp_referenz.split(".")
                self.zp_referenz = f"{rdate[2]}-{rdate[1]}-{rdate[0]}"
        if self.meta.modified:
            self.meta.modified = self.meta.modified.replace("+:", "+00:00")
        if self.meta.created:
            self.meta.created = self.meta.created.replace("+:", "+00:00")

        # Ensure all required fields are present
        return models.Dokument.from_dict({
            "titel": self.meta.title or "Ohne Titel",
            "drucksnr" : self.drucksnr,
            "volltext": self.meta.full_text.strip(),
            "autoren": self.autoren if self.autoren else [],
            "schlagworte": deduplicate(self.schlagworte if self.schlagworte else []),
            "hash": self.meta.hash,
            "zp_modifiziert": datetime.datetime.fromisoformat(self.meta.modified).astimezone(tz=datetime.UTC),
            "zp_created": datetime.datetime.fromisoformat(self.meta.created).astimezone(tz=datetime.UTC),
            "zp_referenz": datetime.datetime.fromisoformat(self.zp_referenz).astimezone(tz=datetime.UTC) if self.zp_referenz else datetime.datetime.fromisoformat(self.meta.created),
            "link": self.url,
            "typ": self.typehint+"",
            "zusammenfassung": self.zusammenfassung.strip() if self.zusammenfassung else None
        })

def deduplicate(ls: list) -> list:
    x = set(ls)
    return list(x)