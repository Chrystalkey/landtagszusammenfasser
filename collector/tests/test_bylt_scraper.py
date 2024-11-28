from collector.scrapers.by_lt import BYLTScraper
from collector.oaiclient.openapi_client import Configuration
import pprint


CURRENT_WP = 19
RESULT_COUNT = 200
listing = f"https://www.bayern.landtag.de/parlament/dokumente/drucksachen?isInitialCheck=0&q=&dknr=&suchverhalten=AND&dokumentenart=Drucksache&ist_basisdokument=on&sort=date&anzahl_treffer={RESULT_COUNT}&wahlperiodeid%5B%5D={CURRENT_WP}&erfassungsdatum%5Bstart%5D=&erfassungsdatum%5Bend%5D=&dokumentenart=Drucksache&suchvorgangsarten%5B%5D=Gesetze%5C%5CGesetzentwurf&suchvorgangsarten%5B%5D=Gesetze%5C%5CStaatsvertrag&suchvorgangsarten%5B%5D=Gesetze%5C%5CHaushaltsgesetz%2C+Nachtragshaushaltsgesetz"

pages = ['https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157725',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157561',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157573',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157414',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157297',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157296',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157159',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157114',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157075',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157006',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157003',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=156976',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=156839',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=156799',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=156761',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=156724',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=156232',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=156233',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=156228',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=155601',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=155566',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=155544',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=155494',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=155505',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=155374',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=155332',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=155232',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=155234',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=154191',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=154177',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=154171',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=153930',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=153934',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=153885',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=153441',
 'https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=153438']

def test_bylt_lextract():
    global listing
    scraper = BYLTScraper(Configuration(host="http://localhost"), [listing])
    urls = scraper.listing_page_extractor(listing)
    assert len(urls) != 0

def test_bylt_pextract():
    global pages
    assert len(pages) != 0
    scraper = BYLTScraper(Configuration(host="http://localhost"), [listing])
    for url in pages:
        gsvh = scraper.page_extractor(url)
        assert gsvh is not None

#def test_bylt_scraper_extract():
#    # Test the BYLTScraper
#    global listing
#    scraper = BYLTScraper(Configuration(host="http://localhost"), [listing])
#    scraper.extract()
#    assert len(scraper.result_objects) != 0
#    print(pprint.pformat(scraper.result_objects))