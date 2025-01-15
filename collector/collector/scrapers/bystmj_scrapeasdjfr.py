from collector.interface import Scraper
from bs4 import BeautifulSoup
from openapi_client import models
import aiohttp
from datetime import datetime as dt_datetime
from datetime import date as dt_date
import datetime # needed for the eval() later
import uuid
import logging
import hashlib
from redis import Redis
import os
import PyPDF2

logger = logging.getLogger(__name__)
class BYSTMJScraper(Scraper):
    def __init__(self, config, session: aiohttp.ClientSession):
        listings = ["https://www.justiz.bayern.de/ministerium/gesetzgebung/"]
        super().__init__(config, uuid.uuid4(), listings, session)
    
    async def listing_page_extractor(self, url: str) -> list:
        logger.debug("BSTMJ::listing_page_extractor")
        base_url = "https://www.justiz.bayern.de"
        async with self.session.get(url) as page_text:
            soup = BeautifulSoup(await page_text.text(), "html.parser")
            cut_ptr = soup.find("div", class_="cut")
            disksoup = cut_ptr.find_all_previous("div", class_="info-box")
            
            result = []
            for disk in disksoup:
                title = disk.find("h2").text
                doclink = base_url+disk.find("li", class_="pdf").find("a")["href"]
                result.append((title, doclink))
            return result

    async def build_document(self, url: str):
        logger.debug("BSTMJ::build_document")
        randid = str(uuid.uuid4())
        logger.debug(f"Extracting PDF Metadata for Url {url}, writing to file {randid}.pdf")
        dic = dict()
        if self.redis.exists(str(url)):
            logger.debug("Cached version found, used")
            dic = eval(self.redis.get(str(url)).decode("utf-8"))
        else:
            logger.debug("Cached version not found, fetching from source")
            async with self.session.get(url) as pdfFile:
                with open(f"{randid}.pdf", "wb") as f:
                    f.write(await pdfFile.read())
                with open(f"{randid}.pdf", "rb") as f:
                    reader = PyPDF2.PdfReader(f)
                    meta = reader.metadata
                    
                    # Handle the date more carefully
                    try:
                        if meta.get('/ModDate'):
                            dt = meta.get('/ModDate')
                        elif meta.get('/CreationDate'):
                            dt = meta.get('/CreationDate')
                        else:
                            dt = dt_datetime.now()
                        
                        # If dt is a string (PDF date format), convert it
                        if isinstance(dt, str):
                            # Remove D: prefix and timezone if present
                            dt = dt.replace('D:', '')[:14]  # Get only the date part
                            dt = dt_datetime.strptime(dt, '%Y%m%d%H%M%S')
                        elif not isinstance(dt, dt_datetime):
                            dt = dt_datetime.now()
                        
                        doc_date = dt
                    except Exception as e:
                        logger.warning(f"Error parsing PDF date: {e}. Using current date.")
                        doc_date = dt_datetime.now()

                    dic = {
                        "title": str(meta.get('/Title', '')),
                        "author": meta.get('/Author', None),
                        "creator": meta.get('/Creator', None),
                        "subject": meta.get('/Subject', None),
                        "lastchange": doc_date.astimezone(datetime.timezone.utc),
                        "hash": hashlib.file_digest(f, "sha256").hexdigest()
                    }
                if os.path.exists(f"{randid}.pdf"):
                    os.remove(f"{randid}.pdf")
            self.redis.set(str(url), str(dic))

        autoren = []
        if dic["author"] is not None:
            autoren.append(f"{dic["author"]} (Author)")
        if dic["creator"] is not None:
            autoren.append(f"{dic["creator"]} (Creator)")

        titel = dic["title"] or dic["subject"] or "Unbekannt"
        dok = models.Dokument.from_dict({
            "titel": titel,
            "last_mod": dic["lastchange"],
            "link": url,
            "hash": dic["hash"],
            "zusammenfassung": "TODO",
            "schlagworte": [],
            "autoren": autoren,
            "typ": "entwurf",
        })
        return dok
    
    # todo: verschlagwortung
    async def item_extractor(self, listing_item: tuple[str, str]) -> models.Gesetzesvorhaben:
        logger.debug("BSTMJ::item_extractor")
        gsvh = models.Gesetzesvorhaben.from_dict({
            "api_id": str(uuid.uuid4()),
            "verfassungsaendernd": False,
            "typ": "bay-parlament",
            "links": [],
            "titel": listing_item[0],
            "initiatoren": ["Bayerisches Staatsministerium der Justiz"],
            "stationen": []
        })
        
        stat = models.Station.from_dict({
            "datum": dt_date.today(),  # Using the document date from build_document
            "gremium": "Bayerisches Staatsministerium der Justiz",
            "parlament": "BY",
            "link": listing_item[1],
            "typ": "preparl-regent",
            "dokumente": [await self.build_document(listing_item[1])],
            "schlagworte": [],
            "stellungnahmen": []
            }
        )

        gsvh.stationen = [stat]
        return gsvh

