import importlib.util
import logging
import os
import time

import aiohttp
import asyncio
from openapi_client import Configuration

from collector.config import CollectorConfiguration
from collector.interface import Scraper

logger = logging.getLogger(__name__)


async def main(config: CollectorConfiguration):
    global logger

    logger.info("Starting new Scraping Cycle")
    # Load all the scrapers from the scrapers dir
    async with aiohttp.ClientSession(
        connector=aiohttp.TCPConnector(limit_per_host=1)
    ) as session:
        scrapers: list[Scraper] = load_scrapers(config, session)
        for scraper in scrapers:
            logger.info(f"Running scraper: {scraper.__class__.__name__}")
            try:
                # Actually run the scraper instance
                await scraper.run()
            except Exception as e:
                logger.error(
                    f"Error while running scraper {scraper.__class__.__name__}: {e}",
                    stack_info=True,
                )


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
                if (
                    isinstance(cls, type)
                    and issubclass(cls, Scraper)
                    and cls is not Scraper
                    and not isinstance(cls, module.__class__)
                ):
                    logger.info(f"Found scraper: {cls.__name__}")
                    scrapers.append(cls(config, session))
    return scrapers


if __name__ == "__main__":
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s | %(levelname)s: \t%(filename)s: \t\t%(message)s",
    )
    logger.info("Starting collector manager.")
    config = CollectorConfiguration(None, None, False)
    logger.info("Configuration Complete")
    CYCLE_TIME = 3 * 60 * 60  # 3 hours
    last_run = None
    while True:
        if last_run is not None and time.time() - last_run < CYCLE_TIME:
            logger.info("Last scraping cycle finished, running again in 3 hours. Bye!")
            time.sleep(CYCLE_TIME - (time.time() - last_run))
            continue
        try:
            last_run = time.time()
            asyncio.run(main(config))
        except KeyboardInterrupt:
            logger.info("Shutting down.")
            break
        except Exception as e:
            logger.error(f"Error: {e}")
            continue
