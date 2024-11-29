from collector.interface import Scraper
from bs4 import BeautifulSoup
from openapi_client import models
import requests
from datetime import date
import uuid

class BYSTMJScraper(Scraper):
    def __init__(self, config):
        listings = ["https://www.justiz.bayern.de/ministerium/gesetzgebung/"]
        super().__init__(config, listings)
    
    def listing_page_extractor(self, url: str) -> list:
        page_text = requests.get(url)
        soup = BeautifulSoup(page_text.content, "html.parser")
        cut_ptr = soup.find("div", class_="cut")
        disksoup = cut_ptr.find_all_previous("div", class_="info-box")
        
        result = []
        for disk in disksoup:
            title = disk.find("h2").text
            doclink = disk.find("li", class_="pdf").find("a")["href"]
            result.append((title, doclink))
        return result
    
    def build_document(self, url: str):
        dok = models.Dokument()
        dok.titel = "TODO"
        dok.zeitpunkt = date.today()
        dok.url = url
        dok.hash = "TODO"
        dok.zusammenfassung = "TODO"
        dok.schlagworte = []
        dok.autoren = ["TODO"]
        dok.typ = "Entwurf"
        return dok
    
    # todo: verschlagwortung
    def item_extractor(self, listing_item: tuple[str, str]) -> models.Gesetzesvorhaben:
        gsvh = models.Gesetzesvorhaben()
        gsvh.api_id = str(uuid.uuid4())
        gsvh.verfassungsaendernd = False
        gsvh.trojaner = False
        gsvh.typ = "landgg"
        gsvh.links = []
        gsvh.titel = listing_item[0]
        gsvh.initiatoren = ["Bayerisches Staatsministerium der Justiz"]
        
        stat = models.Station()
        stat.zeitpunkt = date.today() # fix and extract from pdf
        stat.gremium = "Bayerisches Staatsministerium der Justiz"
        stat.parlament = "BY"
        stat.url = listing_item[1]
        stat.stationstyp = "preparl-regent"
        stat.dokument = self.build_document(listing_item[1])

        gsvh.stationen = [stat]
        return gsvh

