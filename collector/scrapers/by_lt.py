from datetime import date
import pprint
from bs4 import BeautifulSoup
import requests
from typing import Callable
import openapi_client
import openapi_client.models as models
import uuid

class ListingPage:
    url : str = None
    list_extractor: Callable[[str], list[str]] = None
    page_extractor: Callable[[str], models.Gesetzesvorhaben] = None

    def __init__(self, url:str, list_extractor, page_extractor):
        self.url = url
        self.list_extractor = list_extractor
        self.page_extractor = page_extractor

    def fetch_pages(self) -> list[models.Gesetzesvorhaben]:
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

def detect_trojaner(gsvh: models.Gesetzesvorhaben) -> bool:
    print("Warn: Trojaner detection not implemented")
    return False

def create_dokument(url: str, datum) -> models.Dokument:
    dok = models.Dokument()
    dok.zeitpunkt = datum
    dok.titel = "TODO"
    dok.url = url
    dok.hash = "TODO"
    dok.typ = "drucksache"
    dok.zusammenfassung = "TODO"
    
    dok.autoren = []
    return dok

def extract_bylt_vorgangspage(url: str) -> models.Gesetzesvorhaben:
    get_result = requests.get(url)
    soup = BeautifulSoup(get_result.content, "html.parser")
    vorgangs_table = soup.find('tbody', id='vorgangsanzeigedokumente_data')
    rows = vorgangs_table.findAll("tr")
    gsvh = models.Gesetzesvorhaben()
    gsvh.api_id = str(uuid.uuid4())
    gsvh.titel = "TODO"
    gsvh.verfassungsaendernd = False
    gsvh.initiatoren = ["TODO"]
    gsvh.typ = "landgg"
    gsvh.ids = []
    gsvh.links = [url]
    gsvh.stationen = []
    for row in rows:
        cells = row.findAll("td")
        assert len(cells) < 2, f"Warning: Unexpectedly found less than two gridcells in: `{row}`"
        # date is in the first cell
        timestamp = cells[0].text.split(".")
        timestamp = f"{timestamp[2]}-{timestamp[1]}-{timestamp[0]}"
        # content is in the second cell
        stat = models.Station()
        stat.zeitpunkt = timestamp
        stat.parlament = "BY"
        stat.stellungnahmen = []
        cellclass = classify_gridcell(cells[1])
        print(f"Timestamp: {timestamp} Cellclass: {cellclass}")
        if cellclass == "initiativdrucksache":
            link = extract_singlelink(cells[1])
            gsvh.links.append(link)
            stat.stationstyp = "parl-initiativ"
            stat.dokumente = [create_dokument(link, timestamp)]
        elif cellclass == "stellungnahme":
            assert len(gsvh.stationen) > 0, "Error: Stellungnahme ohne Vorhergehenden Gesetzestext"
            stln = models.Stellungnahme()
            stln_urls = extract_schrstellung(cells[1])
            stln.meinung = 0
            stln.dokument = create_dokument(stln_urls["stellungnahme"], timestamp)
            stln.lobbyregister_url = stln_urls["lobbyregister"]
            gsvh.stationen[-1].stellungnahmen.append(stln)
            continue
        elif cellclass == "plenumsdiskussion-uebrw":
            stat.stationstyp = "parl-vollvlsgn"
            stat.dokumente = [create_dokument(extract_plenproto(cells[1]), timestamp)]
        elif cellclass == "plenumsdiskussion-zustm":
            stat.stationstyp = "parl-akzeptanz"
            stat.dokumente = [create_dokument(extract_plenproto(cells[1]), timestamp)]
        elif cellclass == "plenumsdiskussion-ablng":
            stat.stationstyp = "parl-ablehnung"
        elif cellclass == "plenumsbeschluss":
            stat.stationstyp = "parl-schlussab"
            stat.dokumente = [create_dokument(extract_singlelink(cells[1]), timestamp)]
        elif cellclass == "ausschussbericht":
            stat.stationstyp = "parl-ausschber"
            stat.dokumente = [create_dokument(extract_singlelink(cells[1]), timestamp)]
        elif cellclass == "gesetzesblatt":
            stat.stationstyp = "postparl-gsblt"
            stat.dokumente = [create_dokument(extract_singlelink(cells[1]), timestamp)]
        elif cellclass == "unclassified":
            print("Warning: Unclassifiable cell")
        gsvh.stationen.append(stat)
    print(len(rows))
    pprint.pprint(gsvh)
    gsvh.trojaner = detect_trojaner(gsvh)

def extract_singlelink(cellsoup: BeautifulSoup) -> str:
    return cellsoup.find("a")["href"]

# returns: {"typ": link, ...}
def extract_schrstellung(cellsoup: BeautifulSoup) -> dict:
    links = cellsoup.findAll("a")
    return {
        "lobbyregister": links[0]["href"],
        "stellungnahme": links[1]["href"]
    }

def extract_plenproto(cellsoup: BeautifulSoup) -> str:
    cellsoup_ptr = cellsoup.find(string="Protokollauszug")
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
    configuration = openapi_client.Configuration(
        host = "http://localhost"
    )
    with openapi_client.ApiClient(configuration) as api_client:
        api_instance = openapi_client.DefaultApi(api_client)
        gesvh = openapi_client.models.Gesetzesvorhaben()

        api_instance.api_v1_gesetzesvorhaben_post("test_collector", gesvh)
