from abc import ABC, abstractmethod
from typing import Any
import openapi_client
from uuid import UUID
from openapi_client import models
import aiohttp
import asyncio
import logging

logger = logging.getLogger(__name__)
class Scraper(ABC):
    listing_urls : list[str] = []
    result_objects: list[models.Gesetzesvorhaben] = []
    collector_id : UUID = None

    oai_config : openapi_client.Configuration = None

    session : aiohttp.ClientSession = None

    def __init__(self, oai_config: openapi_client.Configuration, collector_id: UUID, listing_urls: list[str], session: aiohttp.ClientSession):
        self.collector_id = collector_id
        self.listing_urls = listing_urls
        self.oai_config = oai_config
        self.result_objects = []
        self.session = session
        global logger
        logger.info(f"Initialized {self.__class__.__name__} with {len(self.listing_urls)} listing urls")
    
    def hash_pdf(self, pdf_url: str) -> str:
        import hashlib
        import requests
        pdf = requests.get(pdf_url)
        return hashlib.sha256(pdf.content).hexdigest()

    def send(self, item: models.Gesetzesvorhaben):
        global logger
        logger.info(f"Sending Item with id `{item.api_id}` to Database")
        logger.debug(f"Collector ID: {self.collector_id}")
        with openapi_client.ApiClient(self.oai_config) as api_client:
            api_instance = openapi_client.DefaultApi(api_client)
            try:
                api_instance.api_v1_gesetzesvorhaben_post(
                    str(self.collector_id), item)
            except openapi_client.ApiException as e:
                logger.error(f"Exception when calling DefaultApi->api_v1_gesetzesvorhaben_post: {e}")
        return item

    """
    for every listing_url in the object
        extract the listing page and then extract the individual pages
        package everything into one or more Gesetzesvorhaben objects and return it
    """
    async def extract(self):
        global logger
        item_list = []
        tasks = []
        for lpage in self.listing_urls:
            logger.debug(f"Initializing listing page extractor for {lpage}")
            tasks.append(self.listing_page_extractor(lpage))
        
        item_list = await asyncio.gather(*tasks)
        iset = set(x for xs in item_list for x in xs)
        tasks = []
        for item in iset:
            logger.debug(f"Initializing item extractor for {item}")
            tasks.append(self.send(await self.item_extractor(item)))

        temp_res = []
        try:
            temp_res = await asyncio.gather(*tasks, return_exceptions=True)
        except Exception as e:
            logger.error(f"Error During Item Extraction: {e}")
        
        for result in temp_res:
            if isinstance(result, models.Gesetzesvorhaben):
                self.result_objects.append(result)
            else:
                logger.error(f"Item extraction failed: {result}")
        logger.info(f"Extractor {self.__class__.__name__} produced {len(self.result_objects)} items")
    
    # extracts the listing page that is behind self.listing_url into the urls of individual pages
    @abstractmethod
    async def listing_page_extractor(self, url: str) -> list:
        assert False, "Warn: Abstact Method Called"

    # extracts the individual pages containing all info into a Gesetzesvorhaben object
    @abstractmethod
    async def item_extractor(self, listing_item) -> models.Gesetzesvorhaben:
        assert False, "Warn: Abstact Method Called"

    