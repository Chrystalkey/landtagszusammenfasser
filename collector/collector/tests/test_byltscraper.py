from collector.scrapers.bylt_scraper import BYLTScraper
from collector.oapicode.openapi_client import Configuration

def test_bylt_listing_extract():
    scraper = BYLTScraper(Configuration(host="http://localhost"))
    urls = scraper.listing_page_extractor(listing)
    assert len(urls) != 0
    for u in pages:
        assert u in urls, "Url `{u}` expected but not found in result set"

def test_bylt_item_extract():
    global pages
    assert len(pages) != 0
    scraper = BYLTScraper(Configuration(host="http://localhost"))
    for url in pages:
        gsvh = scraper.item_extractor(url)
        assert gsvh is not None
        assert len(gsvh.stationen) != 0