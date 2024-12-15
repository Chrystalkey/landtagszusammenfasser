from collector.interface import Scraper
from bs4 import BeautifulSoup
from openapi_client import models
import requests
from datetime import date
import uuid

class BYSTMJScraper(Scraper):
    def __init__(self, config):
        listings = ["https://www.justiz.bayern.de/ministerium/gesetzgebung/"]
        super().__init__(config, uuid.uuid4(), listings)
    
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
        dok = models.Dokument.from_dict({
            "titel" : "TODO",
            "zeitpunkt" : date.today(),
            "url" : url,
            "hash" : "TODO",
            "zusammenfassung" : "TODO",
            "schlagworte" : [],
            "autoren" : ["TODO"],
            "typ" : "entwurf",
        })
        return dok
    
    # todo: verschlagwortung
    def item_extractor(self, listing_item: tuple[str, str]) -> models.Gesetzesvorhaben:
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
            "dokumente" : [self.build_document(listing_item[1])]
            }
        )

        gsvh.stationen = [stat]
        return gsvh

