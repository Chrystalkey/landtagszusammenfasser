from collector.interface import Scraper
import os
import importlib.util
from openapi_client import Configuration
import aiohttp
import asyncio
import logging

scrapers_dir = "./collector/scrapers"
logger = logging.getLogger(__name__)

async def main():
    global logger
    logging.basicConfig(level=logging.INFO)
    logger.info("Starting collector manager.")

    # Load all the scrapers from the scrapers dir
    oapiconfig = Configuration(host=os.environ["LTZF_DATABASE"])

    async with aiohttp.ClientSession(connector=aiohttp.TCPConnector(limit_per_host=1)) as session:
        scrapers: list[Scraper] = load_scrapers(oapiconfig, session)
        for scraper in scrapers:
            logger.info(f"Running scraper: {scraper.__class__.__name__}")
            try:
                # Actually run the scraper instance
                await scraper.extract()

            except Exception as e:
                logger.error(f"Error while running scraper {scraper.__class__.__name__}: {e}")

def load_scrapers(config, session):
    scrapers = []
    for filename in os.listdir(scrapers_dir):
        if filename.endswith("_scraper.py"):
            module_name = filename[:-3]
            module_path = os.path.join(scrapers_dir, filename)
            spec = importlib.util.spec_from_file_location(module_name, module_path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            for attr in dir(module):
                cls = getattr(module, attr)
                if isinstance(cls, type) and issubclass(cls, Scraper) and cls is not Scraper:
                    logger.info(f"Found scraper: {cls.__name__}")
                    scrapers.append(cls(config, session))
    return scrapers

if __name__ == "__main__":
    while True:
        try:
            asyncio.run(main())
        except KeyboardInterrupt:
            logger.info("Shutting down.")
            break
        except Exception as e:
            logger.error(f"Error: {e}")
            continue

