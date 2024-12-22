import importlib.util
import logging
import os
import sys
import time

import aiohttp
import asyncio
from openapi_client import Configuration

from collector.config import CollectorConfiguration
from collector.interface import Scraper

logger = logging.getLogger(__name__)

async def main():
    global logger
    logging.basicConfig(level=logging.INFO)
    logger.info("Starting collector manager.")

    # Load all the scrapers from the scrapers dir
    config = CollectorConfiguration()

    async with aiohttp.ClientSession(connector=aiohttp.TCPConnector(limit_per_host=1)) as session:
        scrapers: list[Scraper] = load_scrapers(config, session)
        for scraper in scrapers:
            logger.info(f"Running scraper: {scraper.__class__.__name__}")
            try:
                # Actually run the scraper instance
                await scraper.extract()

            except Exception as e:
                logger.error(f"Error while running scraper {scraper.__class__.__name__}: {e}", stack_info=True)
                sys.exit(1)

def load_scrapers(config, session):
    scrapers = []
    for filename in os.listdir(config.scrapers_dir):
        if filename.endswith("_scraper.py"):
            module_name = filename[:-3]
            module_path = os.path.join(config.scrapers_dir, filename)
            spec = importlib.util.spec_from_file_location(module_name, module_path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            for attr in dir(module):
                cls = getattr(module, attr)
                if (isinstance(cls, type) and 
                    issubclass(cls, Scraper) and 
                    cls is not Scraper and
                    not isinstance(cls, module.__class__)):
                    logger.info(f"Found scraper: {cls.__name__}")
                    scrapers.append(cls(config, session))
    return scrapers

if __name__ == "__main__":
    while True:
        try:
            while True:
                asyncio.run(main())
                time.sleep(1000)
        except KeyboardInterrupt:
            logger.info("Shutting down.")
            break
        except Exception as e:
            logger.error(f"Error: {e}")
            continue

