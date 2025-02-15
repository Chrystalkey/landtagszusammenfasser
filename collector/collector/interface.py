import logging
from abc import ABC, abstractmethod
from datetime import timedelta
from typing import Any
from uuid import UUID

import aiohttp
import asyncio
from collector.convert import sanitize_for_serialization
from collector.config import CollectorConfiguration

import openapi_client
from openapi_client import models

logger = logging.getLogger(__name__)

class Scraper(ABC):
    listing_urls: list[str] = []
    result_objects: list[models.Gesetzesvorhaben] = []
    collector_id: UUID = None

    config: CollectorConfiguration = None

    session: aiohttp.ClientSession = None
    session_headers: dict[str, str] = {}

    def __init__(
        self,
        config: CollectorConfiguration,
        collector_id: UUID,
        listing_urls: list[str],
        session: aiohttp.ClientSession,
    ):
        self.collector_id = collector_id
        self.listing_urls = listing_urls
        self.config = config
        self.result_objects = []
        self.session = session
        self.session_headers = {}
        global logger
        logger.info (
            f"Initialized {self.__class__.__name__} with {len(self.listing_urls)} listing urls"
        )
        logger.info(f"Set Collector ID to {self.collector_id}")

    async def senditem(self, item: models.Gesetzesvorhaben):
        global logger
        logger.info(f"Sending Item with id `{item.api_id}` to Database")
        logger.debug(f"Collector ID: {self.collector_id}")
        if self.config.api_object_log is not None:
            filepath = self.config.api_object_log / f"{self.collector_id}.json"
            with filepath.open("a", encoding="utf-8") as file:
                file.write(str(sanitize_for_serialization(item)) + ",\n")

        with openapi_client.ApiClient(self.config.oapiconfig) as api_client:
            api_instance = openapi_client.DefaultApi(api_client)
            try:
                ret = api_instance.gsvh_post(str(self.collector_id), item,)
                print(f"Returned: {ret}")
            except openapi_client.ApiException as e:
                logger.error(
                    f"Exception when calling DefaultApi->gsvh_post: {e}"
                )
                if e.status == 422:
                    logger.error("Unprocessable Entity, tried to send item:\n")
                    logger.error(sanitize_for_serialization(item))
        return item

    async def item_processing(self, item):
        return [await self.senditem(await self.item_extractor(item)), item]
    """
    for every listing_url in the object
        extract the listing page and then extract the individual pages
        package everything into one or more Gesetzesvorhaben objects and return it
    """

    async def run(self):
        global logger
        item_list = []
        tasks = []
        logger.debug(f"{self.__class__.__name__}::extract")
        for lpage in self.listing_urls:
            logger.debug(f"Initializing listing page extractor for {lpage}")
            tasks.append(self.listing_page_extractor(lpage))

        item_list = await asyncio.gather(*tasks)
        iset = set(x for xs in item_list for x in xs)
        tasks = []
        tctr = 0
        for item in iset:
            if tctr < 5:
                tctr+=1
            else:
                break
            cached = self.config.cache.get_gsvh(str(item))
            if cached is not None:
                logger.debug(f"URL {item} found in cache, skipping...")
                continue
            logger.debug(f"Initializing item extractor for {item}")
            tasks.append(self.item_processing(item))

        temp_res = []
        try:
            #for r in tasks:
            #    temp_res.append(await r)
            temp_res = await asyncio.gather(*tasks, return_exceptions=True)
        except Exception as e:
            logger.error(f"Error During Item Extraction: {e}", exc_info=True)

        for result in temp_res:
            if not isinstance(result, Exception):
                obj = result[0]
                item = result[1]
                self.result_objects.append(obj)
                self.config.cache.store_gsvh(str(item), obj)
            else:
                logger.error(f"Item extraction failed: {result}", exc_info=True)
        logger.info(
            f"Extractor {self.__class__.__name__} produced {len(self.result_objects)} items"
        )

    # extracts the listing page that is behind self.listing_url into the urls of individual pages
    @abstractmethod
    async def listing_page_extractor(self, url: str) -> list:
        assert False, "Warn: Abstact Method Called"

    # extracts the individual pages containing all info into a Gesetzesvorhaben object
    @abstractmethod
    async def item_extractor(self, listing_item) -> models.Gesetzesvorhaben:
        assert False, "Warn: Abstact Method Called"
