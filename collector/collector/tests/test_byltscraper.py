import asyncio
import jsondiff
from collector.scrapers.bylt_scraper import BYLTScraper, sanitize_orga
from collector.convert import sanitize_for_serialization
from collector.config import CollectorConfiguration
from oapicode.openapi_client import Configuration
import os
import json
import glob
import re
import aiohttp
from oapicode.openapi_client import models

SCRAPER_NAME = "bylt_scraper"


def create_scraper(session):
    global SCRAPER_NAME
    config = CollectorConfiguration(
        api_key="test", openai_api_key="test", testing_mode=True
    )
    config.testing_mode = True
    config.oapiconfig = Configuration(host="http://localhost")
    scraper = BYLTScraper(config, session)
    return scraper


async def inner_bylt_listing_extract():
    async with aiohttp.ClientSession(
        connector=aiohttp.TCPConnector(limit_per_host=1)
    ) as session:
        scraper = create_scraper(session)

        # Find all JSON files in the SCRAPER_NAME subdirectory that start with "listing_"
        test_data_dir = os.path.join(os.path.dirname(__file__), SCRAPER_NAME)

        # Find all JSON files in the directory that start with "listing_"
        listing_files = glob.glob(os.path.join(test_data_dir, "listing_*.json"))

        # Process the first matching file found
        if listing_files:
            with open(listing_files[0], "r", encoding="utf-8") as f:
                listing = json.load(f)
                urls = await scraper.listing_page_extractor(listing.get("listing_url"))
                assert len(urls) >= (listing.get("minimum_count") or 0)
                for url in urls:
                    regex = listing.get("url_regex")
                    if re.fullmatch(regex, url) is None:
                        raise Exception(
                            f"Url `{url}`\n does not match regex \n`{regex}`\n for listing \n`{url}`"
                        )


def json_difference(a, b):
    return jsondiff.diff(
        json.dumps(a, indent=2, ensure_ascii=False),
        json.dumps(b, indent=2, ensure_ascii=False),
    )


async def inner_bylt_item_extract():
    async with aiohttp.ClientSession(
        connector=aiohttp.TCPConnector(limit_per_host=1)
    ) as session:
        scraper = create_scraper(session)

        test_data_dir = os.path.join(os.path.dirname(__file__), SCRAPER_NAME)

        # Find all JSON files in the directory that start with "item_"
        item_files = glob.glob(os.path.join(test_data_dir, "item_*.json"))
        for file in item_files:
            with open(file, "r", encoding="utf-8") as f:
                item_scenario = json.load(f)
                item = models.Vorgang.from_dict(item_scenario.get("result"))
                vg = await scraper.item_extractor(item_scenario.get("url"))
                assert vg is not None
                sanitized_vg = sanitize_for_serialization(vg)
                sanitized_item = sanitize_for_serialization(item)
                dumped = json.dumps(sanitized_vg, indent=2, ensure_ascii=False)
                assert (
                    sanitized_vg == sanitized_item
                ), f"Item `{item_scenario.get('url')}` does not match expected result for scenario `{file}`.\nDifference:\n{json_difference(sanitized_vg, sanitized_item)}\nOutput:\n{dumped}"


def test_bylt_listing_extract():
    asyncio.run(inner_bylt_listing_extract())


def test_bylt_item_extract():
    asyncio.run(inner_bylt_item_extract())
