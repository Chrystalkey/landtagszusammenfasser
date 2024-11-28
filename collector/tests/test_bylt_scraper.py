from collector.scrapers.by_lt import BYLTScraper
from collector.oaiclient.openapi_client import Configuration
import pprint


CURRENT_WP = 19
RESULT_COUNT = 200
listing = f"https://www.bayern.landtag.de/parlament/dokumente/drucksachen?isInitialCheck=0&q=&dknr=&suchverhalten=AND&dokumentenart=Drucksache&ist_basisdokument=on&sort=date&anzahl_treffer={RESULT_COUNT}&wahlperiodeid%5B%5D={CURRENT_WP}&erfassungsdatum%5Bstart%5D=&erfassungsdatum%5Bend%5D=&dokumentenart=Drucksache&suchvorgangsarten%5B%5D=Gesetze%5C%5CGesetzentwurf&suchvorgangsarten%5B%5D=Gesetze%5C%5CStaatsvertrag&suchvorgangsarten%5B%5D=Gesetze%5C%5CHaushaltsgesetz%2C+Nachtragshaushaltsgesetz"

def test_bylt_lextract():
    global listing
    scraper = BYLTScraper(Configuration(host="http://localhost"), [listing])
    urls = scraper.listing_page_extractor(listing)
    assert len(urls) != 0
    print(urls)

def test_bylt_pextract():
    global listing
    scraper = BYLTScraper(Configuration(host="http://localhost"), [listing])
    urls = scraper.listing_page_extractor(listing)
    assert len(urls) != 0
    for url in urls:
        gsvh = scraper.page_extractor(url)
        assert gsvh is not None
        print(pprint.pformat(gsvh))
        print()
        print()
    assert False

#def test_bylt_scraper_extract():
#    # Test the BYLTScraper
#    global listing
#    scraper = BYLTScraper(Configuration(host="http://localhost"), [listing])
#    scraper.extract()
#    assert len(scraper.result_objects) != 0
#    print(pprint.pformat(scraper.result_objects))