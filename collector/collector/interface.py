import logging
from abc import ABC, abstractmethod
from datetime import timedelta
from typing import Any
from uuid import UUID

import aiohttp
import asyncio
from redis import Redis

import openapi_client
from openapi_client import models
from collector.config import CollectorConfiguration

logger = logging.getLogger(__name__)


class Scraper(ABC):
    listing_urls: list[str] = []
    result_objects: list[models.Gesetzesvorhaben] = []
    collector_id: UUID = None

    config: CollectorConfiguration = None

    session: aiohttp.ClientSession = None
    redis: Redis = None

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
        self.redis = Redis(host=config.redis_host, port=config.redis_port)
        global logger
        logger.info(
            f"Initialized {self.__class__.__name__} with {len(self.listing_urls)} listing urls"
        )
        logger.info(f"Set Collector ID to {self.collector_id}")

    async def send(self, item: models.Gesetzesvorhaben):
        global logger
        logger.info(f"Sending Item with id `{item.api_id}` to Database")
        logger.debug(f"Collector ID: {self.collector_id}")
        if self.config.api_object_log is not None:
            filepath = self.config.api_object_log / f"{self.collector_id}.json"
            with filepath.open("a", encoding="utf-8") as file:
                file.write(str(item.to_str()) + ",\n")

        with openapi_client.ApiClient(self.config.oapiconfig) as api_client:
            api_instance = openapi_client.DefaultApi(api_client)
            try:
                api_instance.api_v1_gesetzesvorhaben_post(str(self.collector_id), item)
            except openapi_client.ApiException as e:
                logger.error(
                    f"Exception when calling DefaultApi->api_v1_gesetzesvorhaben_post: {e}"
                )
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
        logger.debug(f"{self.__class__.__name__}::extract")
        for lpage in self.listing_urls:
            logger.debug(f"Initializing listing page extractor for {lpage}")
            tasks.append(self.listing_page_extractor(lpage))

        item_list = await asyncio.gather(*tasks)
        iset = set(x for xs in item_list for x in xs)
        tasks = []
        for item in iset:
            if self.redis.exists(str(item)):
                logger.info(f"URL {item} found in cache, skipping...")
                continue
            self.redis.setex(str(item), timedelta(minutes=12), value="{}")
            logger.debug(f"Initializing item extractor for {item}")
            tasks.append(self.send(await self.item_extractor(item)))

        temp_res = []
        try:
            temp_res = await asyncio.gather(*tasks, return_exceptions=True)
        except Exception as e:
            logger.error(f"Error During Item Extraction: {e}", exc_info=True)

        for result in temp_res:
            if isinstance(result, models.Gesetzesvorhaben):
                self.result_objects.append(result)
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
