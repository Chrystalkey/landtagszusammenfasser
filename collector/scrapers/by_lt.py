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
- ausschussbericht
- plenumsdiskussion-uebrw
- plenumsdiskussion-zustm
- plenumsdiskussion-ablng
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
        if cellsoup.text.find("Überweisung") != -1:
            return "plenumsdiskussion-uebrw"
        elif cellsoup.text.find("Zustimmung") != -1:
            return "plenumsdiskussion-zustm"
        elif cellsoup.text.find("Ablehnung") != -1:
            return "plenumsdiskussion-ablng"
        else:
            print(f"Warning: Plenumsdiskussion without specific classification: `{cellsoup}`")
            return "unclassified"
    elif cellsoup.text.find("Plenum") != -1 and cellsoup.text.find("Plenarprotokoll") == -1:
        return "plenumsbeschluss"
    elif cellsoup.text.find("Ausschuss") != -1:
        return "ausschussbericht"
    elif cellsoup.text.find("Gesetz- und Verordnungsblatt") != -1:
        return "gesetzesblatt"
    else:
        return "unclassified"

def extract_bylt_vorgangspage(url: str) -> gesetzesvorhaben.Gesetzesvorhaben:
    get_result = requests.get(url)
    soup = BeautifulSoup(get_result.content, "html.parser")
    vorgangs_table = soup.find('tbody', id='vorgangsanzeigedokumente_data')
    rows = vorgangs_table.findAll("tr")
    for row in rows:
        cells = row.findAll("td")
        if len(cells) < 2:
            print(f"Warning: Unexpectedly found less than two gridcells in: `{row}`")
            continue
        # date is in the first cell
        timestamp = cells[0].text
        # content is in the second cell
        cellclass = classify_gridcell(cells[1])
        print(f"Timestamp: {timestamp} Cellclass: {cellclass}")
        match cellclass:
            case "initiativdrucksache": print(extract_singlelink(cells[1]))
            case "stellungnahme": print(extract_schrstellung(cells[1]))
            case "plenumsdiskussion-uebrw": print(extract_plenproto(cells[1]))
            case "plenumsdiskussion-zustm": print(extract_plenproto(cells[1]))
            case "plenumsdiskussion-ablng": print(extract_plenproto(cells[1]))
            case "plenumsbeschluss": print(extract_singlelink(cells[1]))
            case "ausschussbericht": print(extract_singlelink(cells[1]))
            case "gesetzesblatt": print(extract_gbl_ausz(cells[1]))
            case "unclassified": print("Unclassifiable cell")
    # TODO: put this into a Gesetzesvorhaben object
    print(len(rows))

def extract_singlelink(cellsoup: BeautifulSoup) -> str:
    return cellsoup.find("a")["href"]

# returns: {"typ": [links, links, links], ...}
def extract_schrstellung(cellsoup: BeautifulSoup) -> dict:
    links = cellsoup.findAll("a")
    return {
        "lobbyregister": [links[0]["href"]],
        "stellungnahme": [links[1]["href"]]
    }

def extract_plenproto(cellsoup: BeautifulSoup) -> str:
    cellsoup_ptr = cellsoup.find(text="Protokollauszug")
    cellsoup_ptr = cellsoup_ptr.find_previous("br")
    return cellsoup_ptr.find_next("a")["href"]

def extract_gbl_ausz(cellsoup: BeautifulSoup) -> str:
    return cellsoup.findAll("a")[1]["href"]

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