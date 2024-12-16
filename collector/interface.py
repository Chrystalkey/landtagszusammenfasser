from abc import ABC, abstractmethod
from typing import Any
import openapi_client
from uuid import UUID
from openapi_client import models

class Scraper(ABC):
    listing_urls : list[str] = []
    result_objects: list[models.Gesetzesvorhaben] = []
    collector_id : UUID = None

    oai_config : openapi_client.Configuration = None

    def __init__(self, oai_config: openapi_client.Configuration, collector_id: UUID, listing_urls: list[str]):
        self.collector_id = collector_id
        self.listing_urls = listing_urls
        self.oai_config = oai_config
        self.result_objects = []
    
    def hash_pdf(self, pdf_url: str) -> str:
        import hashlib
        import requests
        pdf = requests.get(pdf_url)
        return hashlib.sha256(pdf.content).hexdigest()

    def send(self):
        with openapi_client.ApiClient(self.oai_config) as api_client:
            api_instance = openapi_client.DefaultApi(api_client)
            print(f"Sending {len(self.result_objects)} entries to API")
            print("Collector ID: " + str(self.collector_id))
            for gsvh in self.result_objects:
                try:
                    api_instance.api_v1_gesetzesvorhaben_post(
                        str(self.collector_id),
                        gsvh)
                except openapi_client.ApiException as e:
                    print("Exception when calling DefaultApi->api_v1_gesetzesvorhaben_post: %s\n" % e)

    """
    for every listing_url in the object
        extract the listing page and then extract the individual pages
        package everything into one or more Gesetzesvorhaben objects and return it
    """
    def extract(self):
        item_list = []
        for lpage in self.listing_urls:
            print(lpage)
            item_list.extend(self.listing_page_extractor(lpage))
        iset = set(item_list)
        
        for item in iset:
            print(f"`{item}`")
            self.result_objects.append(self.item_extractor(item))
    
    # extracts the listing page that is behind self.listing_url into the urls of individual pages
    @abstractmethod
    def listing_page_extractor(self, url: str) -> list:
        assert False, "Warn: Abstact Method Called"

    # extracts the individual pages containing all info into a Gesetzesvorhaben object
    @abstractmethod
    def item_extractor(self, listing_item) -> models.Gesetzesvorhaben:
        assert False, "Warn: Abstact Method Called"

    