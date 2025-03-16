from abc import ABC
from oapicode.openapi_client import models
from uuid import UUID,uuid4
from kreuzberg import extract_file
from collector.config import Configuration
import logging
import os
import json

logger = logging.getLogger(__name__)

class Document(ABC):
    titel = None
    kurztitel=None
    zusammenfassung=None
    volltext=None
    schlagworte=[]
    autoren=[]
    autorpersonen=[]
    hash=None
    last_mod=None
    typ=None
    herausgeber=None
    fileid=None
    link=None
    drucks_nr=None
    meinung=None
    session = None
    config = config

    def __init__(self, typhint: models.Doktyp, session, config : Configuration):
        self.typ=typhint
        self.fileid = UUID("00000000-0000-0000-0000-000000000000") if config.testing_mode else uuid4()
        self.session = session
        self.config = config

        self.titel = None
        self.kurztitel=None
        self.zusammenfassung=None
        self.volltext=None
        self.schlagworte=[]
        self.autoren=[]
        self.autorpersonen=[]
        self.hash=None
        self.last_mod=None
        self.herausgeber=None
        self.link=None
        self.drucks_nr=None
        self.meinung=None
        self.download_extract()
        self.semantic_extraction()

    ### outputs full text
    async def download_extract(self):        
        logger.info(f"Downloading document from {self.link}")
        try:
            async with self.session.get(self.link) as response:
                if response.status != 200:
                    raise Exception(f"Failed to download document, status: {response.status}")
                
                with open(f"{self.fileid}.pdf", "wb") as f:
                    f.write(await response.read())
            if not os.path.exists(f"{self.fileid}.pdf") or os.path.getsize(f"{self.fileid}.pdf") == 0:
                raise Exception("Downloaded file is empty or doesn't exist")
            ### extraction
            result = await extract_file(f"{self.fileid}.pdf", language="deu", max_processes=6)
            self.volltext = result.content
            ### end of extraction
        except Exception as e:
            logger.error(f"Download error for {self.url}: {e}")
            raise
    
    @classmethod
    def from_dict(cls, d: dict):
        assert False, "TODO"

    def to_dict(self) ->dict:
        assert False, "TODO"

    @classmethod
    def from_json(cls, string: str):
        return cls.from_dict(json.loads(string))
    
    def to_json(self) -> str:
        json.dumps(self.to_dict())

    ## expects a filled full text as described above and fills out all other properties to full satisfaction.
    ## this is to be done by the subclassses
    def semantic_extraction(self):
        assert False, "Warn: Abstract Method Called"

    ## packages the thing up as an API Document
    def package() -> models.Dokument:
        assert False, "Warn: Abstract Method Called"