from bs4 import BeautifulSoup
import requests
from models import gesetzesvorhaben
from typing import Callable

class ListingPage:
    url : str = None
    list_extractor: Callable[[str], list[str]] = None
    page_extractor: Callable[[str], gesetzesvorhaben.Gesetzesvorhaben] = None

    def __init__(self, url:str, list_extractor, page_extractor):
        self.url = url
        self.list_extractor = list_extractor
        self.page_extractor = page_extractor

    def fetch_pages(self) -> list[gesetzesvorhaben.Gesetzesvorhaben]:
        urls = self.list_extractor(self.url)
        contents = []
        for u in urls:
            contents.append(self.page_extractor(u))
        return contents

class BYLTScraper:
    listing_pages = []

    def __init__(self, listing_pages):
        self.listing_pages = listing_pages
    
    def run(self) -> list:
        contents = []
        for lp in self.listing_pages:
            contents.extend(lp.fetch_pages())
        return contents
    
    def update_database(self):
        contents = self.run()
        for c in contents:
            print("Updating DB with thing")
            pass

def extract_bylt_resultpage(url: str):
    # assumes a full page without pagination
    get_result = requests.get(url)
    soup = BeautifulSoup(get_result.content, "html.parser")
    # finds all result boxes
    resdiv_soup = soup.find_all("div", class_="row result")
    vgpage_urls = []
    for div in resdiv_soup:
        inner_div = div.find("div")
        heading = inner_div.find("h5")
        a_tags = inner_div.find_all("a", class_="link-with-icon")
        for a in a_tags:
            if "views/vorgangsanzeige" in a["href"]:
                vgpage_urls.append(a["href"])
    return vgpage_urls

"""
classes:
- initiativdrucksache
- stellungnahme
- plenumsdiskussion
- ausschussbericht
- plenumsbeschluss
- gesetzesblatt
- unclassified
returns (class, link to document)
"""
def classify_gridcell(cellsoup: BeautifulSoup) -> str:
    if cellsoup.text.find("Initiativdrucksache") != -1:
        return "initiativdrucksache"
    elif cellsoup.text.find("Schriftliche Stellungnahmen im Gesetzgebungsverfahren")  != -1:
        return "stellungnahme"
    elif cellsoup.text.find("Plenum") != -1 and cellsoup.text.find("Plenarprotokoll") != -1:
        return "plenumsdiskussion"
    elif cellsoup.text.find("Plenum") != -1 and cellsoup.text.find("Plenarprotokoll") == -1:
        return "plenumsbeschluss"
    elif cellsoup.text.find("Ausschuss") != -1:
        return "ausschussbericht"
    elif cellsoup.text.find("Gesetz- und Verordnungsblatt") != -1:
        return "gesetzesblatt"
    else:
        return "unclassified"

def extract_bylt_vorgangspage(url: str):
    get_result = requests.get(url)
    soup = BeautifulSoup(get_result.content, "html.parser")
    vorgangs_table = soup.find('tbody', id='vorgangsanzeigedokumente_data')
    rows = vorgangs_table.findAll("tr")
    for row in rows:
        cells = row.findAll("td")
        if len(cells) < 2:
            print(f"Warning: Unexpectedly found less than two gridcells in: `{row}`")
            continue
        # print date
        timestamp = cells[0].text
        cellclass = classify_gridcell(cells[1])
        print(f"Timestamp: {timestamp} Cellclass: {cellclass}")

    print(len(rows))


if __name__ == "__main__":
    #CURRENT_WP = 19
    #RESULT_COUNT = 200
    #collector = BYLTScraper([
    #        ListingPage(f"https://www.bayern.landtag.de/parlament/dokumente/drucksachen?isInitialCheck=0&q=&dknr=&suchverhalten=AND&dokumentenart=Drucksache&ist_basisdokument=on&sort=date&anzahl_treffer={RESULT_COUNT}&wahlperiodeid%5B%5D={CURRENT_WP}&erfassungsdatum%5Bstart%5D=&erfassungsdatum%5Bend%5D=&dokumentenart=Drucksache&suchvorgangsarten%5B%5D=Gesetze%5C%5CGesetzentwurf&suchvorgangsarten%5B%5D=Gesetze%5C%5CStaatsvertrag&suchvorgangsarten%5B%5D=Gesetze%5C%5CHaushaltsgesetz%2C+Nachtragshaushaltsgesetz",
    #                    extract_bylt_resultpage,
    #                    extract_bylt_page
    #                    )
    #    ])
    #urls = extract_bylt_resultpage("https://www.bayern.landtag.de/parlament/dokumente/drucksachen?isInitialCheck=0&q=&dknr=&suchverhalten=AND&dokumentenart=Drucksache&ist_basisdokument=on&sort=date&anzahl_treffer=200&wahlperiodeid%5B%5D=19&erfassungsdatum%5Bstart%5D=&erfassungsdatum%5Bend%5D=&dokumentenart=Drucksache&suchvorgangsarten%5B%5D=Gesetze%5C%5CGesetzentwurf&suchvorgangsarten%5B%5D=Gesetze%5C%5CStaatsvertrag&suchvorgangsarten%5B%5D=Gesetze%5C%5CHaushaltsgesetz%2C+Nachtragshaushaltsgesetz")
    #print(urls)
    #print(len(urls))
##
    #URL = urls[0]
    #print(extract_bylt_page(URL))
    extract_bylt_vorgangspage("https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=155494")