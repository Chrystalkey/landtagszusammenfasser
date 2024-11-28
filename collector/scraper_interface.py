from abc import ABC, abstractmethod
from typing import Any
import openapi_client
from openapi_client import models

class Scraper(ABC):
    listing_urls : list[str] = []
    result_objects: list[models.Gesetzesvorhaben] = []

    oai_config : openapi_client.Configuration = None

    def __init__(self, oai_config: openapi_client.Configuration, listing_urls: list[str]):
        self.listing_url = listing_urls
        self.oai_config = oai_config
    
    def send(self):
        with openapi_client.ApiClient(self.oai_config) as api_client:
            api_instance = openapi_client.GesetzesvorhabenApi(api_client)
            for gsvh in self.result_objects:
                api_instance.api_v1_gesetzesvorhaben_post(gsvh)

    """
    for every listing_url in the object
        extract the listing page and then extract the individual pages
        package everything into one or more Gesetzesvorhaben objects and return it
    """
    @abstractmethod
    def extract(self):
        pass
    
    # extracts the listing page that is behind self.listing_url into the urls of individual pages
    @abstractmethod
    def listing_page_extractor(self, url: str) -> list[str]:
        pass

    # extracts the individual pages containing all info into a Gesetzesvorhaben object
    @abstractmethod
    def page_extractor(self, url: str) -> models.Gesetzesvorhaben:
        pass

    # classifies an object that comes in. Can be a station or be a `stellungnahme`
    # and can be a beautifulsoup, a selenium object, text or whatever else
    @abstractmethod
    def classify_object(self, context) -> str:
        pass

    # create a document object from a url
    @abstractmethod
    def create_document(self, url: str) -> models.Dokument:
        pass