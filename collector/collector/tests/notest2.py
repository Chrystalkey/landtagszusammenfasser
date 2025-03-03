from collector.collector.scrapers.bystmj_scrapeasdjfr import BYSTMJScraper
from collector.oapicode.openapi_client import Configuration

def test_pextractor():
    scraper = BYSTMJScraper(Configuration(host="http://localhost:8080"))
    for item in [("Testsubject", "http://example.com")]:
        res = scraper.item_extractor(item)
        assert res is not None

def test_lextractor():
    scraper = BYSTMJScraper(Configuration(host="http://localhost:8080"))
    items = scraper.listing_page_extractor(scraper.listing_urls[0])
    assert len(items) > 0