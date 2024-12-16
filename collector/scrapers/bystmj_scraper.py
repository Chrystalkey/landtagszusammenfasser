from collector.interface import Scraper
from bs4 import BeautifulSoup
from openapi_client import models
import aiohttp
from datetime import date
import uuid
import logging

logger = logging.getLogger(__name__)
class BYSTMJScraper(Scraper):
    def __init__(self, config, session: aiohttp.ClientSession):
        listings = ["https://www.justiz.bayern.de/ministerium/gesetzgebung/"]
        super().__init__(config, uuid.uuid4(), listings, session)
    
    async def listing_page_extractor(self, url: str) -> list:
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
        dok = models.Dokument.from_dict({
            "titel" : "TODO",
            "zeitpunkt" : date.today(),
            "url" : url,
            "hash" : self.hash_pdf(url),
            "zusammenfassung" : "TODO",
            "schlagworte" : [],
            "autoren" : ["TODO"],
            "typ" : "entwurf",
        })
        return dok
    
    # todo: verschlagwortung
    async def item_extractor(self, listing_item: tuple[str, str]) -> models.Gesetzesvorhaben:
        gsvh = models.Gesetzesvorhaben.from_dict({
            "api_id" : str(uuid.uuid4()),
            "verfassungsaendernd" : False,
            "typ" : "landgg",
            "links" : [],
            "titel" : listing_item[0],
            "initiatoren" : ["Bayerisches Staatsministerium der Justiz"],
            "stationen": []
        })
        
        stat = models.Station.from_dict({
            "zeitpunkt" : date.today(), # fix and extract from pdf
            "gremium" : "Bayerisches Staatsministerium der Justiz",
            "parlament" : "BY",
            "url" : listing_item[1],
            "typ" : "preparl-regent",
            "dokumente" : [await self.build_document(listing_item[1])]
            }
        )

        gsvh.stationen = [stat]
        return gsvh

