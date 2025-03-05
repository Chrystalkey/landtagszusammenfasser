import logging
from abc import ABC, abstractmethod
from datetime import timedelta
from typing import Any, List, Optional
from uuid import UUID
from pathlib import Path

import aiohttp
import asyncio
from collector.convert import sanitize_for_serialization
from collector.config import CollectorConfiguration

import openapi_client
from openapi_client import models

logger = logging.getLogger(__name__)

class Scraper(ABC):
    listing_urls: List[str] = []
    result_objects: List[models.Vorgang] = []
    collector_id: UUID = None

    config: CollectorConfiguration = None

    session: aiohttp.ClientSession = None
    session_headers: dict[str, str] = {}

    def __init__(
        self,
        config: CollectorConfiguration,
        collector_id: UUID,
        listing_urls: List[str],
        session: aiohttp.ClientSession,
    ):
        self.collector_id = collector_id
        self.listing_urls = listing_urls
        self.config = config
        self.result_objects = []
        self.session = session
        self.session_headers = {}
        global logger
        logger.info(
            f"Initialized {self.__class__.__name__} with {len(self.listing_urls)} listing urls"
        )
        logger.info(f"Set Collector ID to {self.collector_id}")

    async def senditem(self, item: models.Vorgang) -> Optional[models.Vorgang]:
        """
        Send a Vorgang item to the API
        
        Args:
            item: The Vorgang object to send
            
        Returns:
            The sent item on success, None on failure
        """
        global logger
        logger.info(f"Sending Item with id `{item.api_id}` to Database")
        logger.debug(f"Collector ID: {self.collector_id}")
        
        # Save to log file if configured
        if self.config.api_object_log is not None:
            try:
                filepath = Path(self.config.api_object_log) / f"{self.collector_id}.json"
                with filepath.open("a", encoding="utf-8") as file:
                    file.write(str(sanitize_for_serialization(item)) + ",\n")
            except Exception as e:
                logger.error(f"Failed to write to API object log: {e}")

        # Send to API
        with openapi_client.ApiClient(self.config.oapiconfig) as api_client:
            api_instance = openapi_client.DefaultApi(api_client)
            try:
                # Note: Changed from gsvh_post to vorgang_put to match API spec
                ret = api_instance.vorgang_put(str(self.collector_id), item)
                logger.info(f"API Response: {ret}")
                return item
            except openapi_client.ApiException as e:
                logger.error(f"API Exception: {e}")
                if e.status == 422:
                    logger.error("Unprocessable Entity, tried to send item:\n")
                    logger.error(sanitize_for_serialization(item))
                    try:
                        filepath = Path(self.config.api_object_log or "locallogs") / f"{self.collector_id}.json"
                        with filepath.open("a", encoding="utf-8") as file:
                            file.write(str(sanitize_for_serialization(item)) + ",\n")
                    except Exception as e:
                        logger.error(f"Failed to write to API object log: {e}")
                elif e.status == 401:
                    logger.error("Authentication failed. Check your API key.")
                elif e.status == 409:
                    logger.error(f"Conflict: Item with ID {item.api_id} already exists")
                return None
            except Exception as e:
                logger.error(f"Unexpected error sending item to API: {e}")
                return None

    async def item_processing(self, item):
        """Process an item by extracting and sending it to the API"""
        try:
            extracted_item = await self.item_extractor(item)
            sent_item = await self.senditem(extracted_item)
            return [sent_item, item]
        except Exception as e:
            logger.error(f"Error processing item {item}: {e}", exc_info=True)
            raise
    
    async def run(self):
        """
        Main method to run the scraper:
        1. Extract all listing pages
        2. Extract individual items
        3. Send items to API
        4. Store in cache
        """
        global logger
        item_list = []
        tasks = []
        logger.debug(f"{self.__class__.__name__}::extract")
        
        # Extract all listing pages
        try:
            for lpage in self.listing_urls:
                logger.debug(f"Initializing listing page extractor for {lpage}")
                tasks.append(self.listing_page_extractor(lpage))
            
            # Wait for all listing page extractor tasks to complete
            item_list = await asyncio.gather(*tasks, return_exceptions=True)
            
            # Handle any exceptions from listing page extractors
            for i, result in enumerate(item_list):
                if isinstance(result, Exception):
                    logger.error(f"Error extracting listing page {self.listing_urls[i]}: {result}")
                    item_list[i] = []  # Replace exception with empty list
            
            # Flatten the list of lists into a set to eliminate duplicates
            iset = set(x for xs in item_list if isinstance(xs, list) for x in xs)
        except Exception as e:
            logger.error(f"Error extracting listing pages: {e}", exc_info=True)
            return
        
        # Process all items
        tasks = []
        processed_count = 0
        skipped_count = 0
        
        for item in iset:
            # Check if item is already in cache
            cached = self.config.cache.get_gsvh(str(item))
            if cached is not None:
                logger.debug(f"URL {item} found in cache, skipping...")
                skipped_count += 1
                continue
                
            logger.debug(f"Initializing item extractor for {item}")
            tasks.append(self.item_processing(item))
            processed_count += 1

        logger.info(f"Processing {processed_count} items, skipped {skipped_count} cached items")
        
        # Process all items
        temp_res = []
        if tasks:
            try:
                temp_res = await asyncio.gather(*tasks, return_exceptions=True)
            except Exception as e:
                logger.error(f"Error during item extraction: {e}", exc_info=True)

        # Process results and store in cache
        success_count = 0
        error_count = 0
        
        for result in temp_res:
            if not isinstance(result, Exception) and result and result[0]:
                obj = result[0]
                item = result[1]
                self.result_objects.append(obj)
                self.config.cache.store_gsvh(str(item), obj)
                success_count += 1
            else:
                error_count += 1
                if isinstance(result, Exception):
                    logger.error(f"Item extraction failed with exception: {result}", exc_info=True)
                else:
                    logger.error(f"Item extraction failed with result: {result}")

        logger.info(
            f"Extractor {self.__class__.__name__} completed: {success_count} successes, {error_count} errors"
        )

    # extracts the listing page that is behind self.listing_url into the urls of individual pages
    @abstractmethod
    async def listing_page_extractor(self, url: str) -> List[str]:
        """
        Extract a listing page into individual item URLs
        
        Args:
            url: The listing page URL
            
        Returns:
            A list of item URLs found on the listing page
        """
        assert False, "Warn: Abstract Method Called"

    # extracts the individual pages containing all info into a Vorgang object
    @abstractmethod
    async def item_extractor(self, listing_item) -> models.Vorgang:
        """
        Extract an individual item into a Vorgang object
        
        Args:
            listing_item: The item URL or identifier
            
        Returns:
            A Vorgang object containing the extracted information
        """
        assert False, "Warn: Abstract Method Called"
