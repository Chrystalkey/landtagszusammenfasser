import pprint
from bs4 import BeautifulSoup
import requests
import openapi_client.models as models
from datetime import date
import uuid

from collector.interface import Scraper

class BYLTScraper(Scraper):
    def __init__(self, config):
        CURRENT_WP = 19
        RESULT_COUNT = 200
        listing_urls = [
            f"https://www.bayern.landtag.de/parlament/dokumente/drucksachen?isInitialCheck=0&q=&dknr=&suchverhalten=AND&dokumentenart=Drucksache&ist_basisdokument=on&sort=date&anzahl_treffer={RESULT_COUNT}&wahlperiodeid%5B%5D={CURRENT_WP}&erfassungsdatum%5Bstart%5D=&erfassungsdatum%5Bend%5D=&dokumentenart=Drucksache&suchvorgangsarten%5B%5D=Gesetze%5C%5CGesetzentwurf&suchvorgangsarten%5B%5D=Gesetze%5C%5CStaatsvertrag&suchvorgangsarten%5B%5D=Gesetze%5C%5CHaushaltsgesetz%2C+Nachtragshaushaltsgesetz"
        ]
        super().__init__(config, listing_urls)

    def listing_page_extractor(self, url) -> list[str]:
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
        assert len(vgpage_urls) != 0, "Error: No Entry extracted from listing page"
        return vgpage_urls

    def item_extractor(self, listing_item) -> models.Gesetzesvorhaben:
        get_result = requests.get(listing_item)
        print(f"Extracting from {listing_item}")
        soup = BeautifulSoup(get_result.content, "html.parser")
        vorgangs_table = soup.find('tbody', id='vorgangsanzeigedokumente_data')
        rows = vorgangs_table.findAll("tr")
        gsvh = models.Gesetzesvorhaben()
        gsvh.api_id = str(uuid.uuid4())
        gsvh.titel = "TODO"
        gsvh.verfassungsaendernd = False
        gsvh.typ = "landgg"
        gsvh.ids = []
        gsvh.links = [listing_item]
        gsvh.stationen = []
        
        # Initiatoren
        init_ptr = soup.find(string="Initiatoren")
        initiat_lis = init_ptr.find_next("ul").findAll("li")
        gsvh.initiatoren = []
        for ini in initiat_lis: 
            gsvh.initiatoren.append(ini.text)
        assert len(gsvh.initiatoren) > 0, f"Error: Could not find Initiatoren for url {listing_item}"

        for row in rows:
            cells = row.findAll("td")

            assert len(cells) == 2, f"Warning: Unexpectedly found more or less than exactly two gridcells in: `{row}` of url `{listing_item}`"
            
            # date is in the first cell
            if cells[0].text == "Beratung / Ergebnis folgt":
                continue
            timestamp = cells[0].text.split(".")
            assert len(timestamp) == 3, f"Error: Unexpected date format: `{timestamp}` of url `{listing_item}`"
            timestamp = date(int(timestamp[2]), int(timestamp[1]), int(timestamp[0]))
            # content is in the second cell
            stat = models.Station()
            stat.zeitpunkt = timestamp
            stat.parlament = "BY"
            stat.stellungnahmen = []
            cellclass = self.classify_object(cells[1])
            print(f"Timestamp: {timestamp} Cellclass: {cellclass}")
            if cellclass == "initiativdrucksache":
                link = extract_singlelink(cells[1])
                gsvh.links.append(link)
                stat.stationstyp = "parl-initiativ"
                stat.dokumente = [self.create_document(link)]
            elif cellclass == "stellungnahme":
                assert len(gsvh.stationen) > 0, "Error: Stellungnahme ohne Vorhergehenden Gesetzestext"
                stln = models.Stellungnahme()
                stln_urls = extract_schrstellung(cells[1])
                stln.meinung = 0
                stln.dokument = self.create_document(stln_urls["stellungnahme"])
                stln.lobbyregister_url = stln_urls["lobbyregister"]
                gsvh.stationen[-1].stellungnahmen.append(stln)
                continue
            elif cellclass == "plenumsdiskussion-uebrw":
                stat.stationstyp = "parl-vollvlsgn"
                stat.dokumente = [self.create_document(extract_plenproto(cells[1]))]
            elif cellclass == "plenumsdiskussion-zustm":
                stat.stationstyp = "parl-akzeptanz"
                stat.dokumente = [self.create_document(extract_plenproto(cells[1]))]
            elif cellclass == "plenumsdiskussion-ablng":
                stat.stationstyp = "parl-ablehnung"
            elif cellclass == "plenumsbeschluss":
                stat.stationstyp = "parl-schlussab"
                stat.dokumente = [self.create_document(extract_singlelink(cells[1]))]
            elif cellclass == "ausschussbericht":
                stat.stationstyp = "parl-ausschber"
                stat.dokumente = [self.create_document(extract_singlelink(cells[1]))]
            elif cellclass == "gesetzesblatt":
                stat.stationstyp = "postparl-gsblt"
                stat.dokumente = [self.create_document(extract_singlelink(cells[1]))]
            elif cellclass == "unclassified":
                print("Warning: Unclassifiable cell")
            gsvh.stationen.append(stat)
        print(len(rows))
        pprint.pprint(gsvh)
        gsvh.trojaner = detect_trojaner(gsvh)
        return gsvh

    def create_document(self, url: str) -> models.Dokument:
        dok = models.Dokument()
        dok.zeitpunkt = date.today()
        dok.titel = "TODO"
        dok.url = url
        dok.hash = "TODO"
        dok.typ = "drucksache"
        dok.zusammenfassung = "TODO"
        
        dok.autoren = []
        return dok
    
    def classify_object(self, context) -> str:
        cellsoup = context
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

def extract_singlelink(cellsoup: BeautifulSoup) -> str:
    return cellsoup.find("a")["href"]

# returns: {"typ": link, ...}
def extract_schrstellung(cellsoup: BeautifulSoup) -> dict:
    links = cellsoup.findAll("a")
    assert len(links) > 0 and len(links) < 3, f"Error: Unexpected number of links in Stellungnahme: {len(links)}, in cellsoup `{cellsoup}`"
    if len(links) == 2:
        return {
        "lobbyregister": links[0]["href"],
        "stellungnahme": links[1]["href"]
        }
    elif len(links) == 1:
        return {
        "stellungnahme": links[0]["href"],
        "lobbyregister": ""
        }

def extract_plenproto(cellsoup: BeautifulSoup) -> str:
    cellsoup_ptr = cellsoup.find(string="Protokollauszug")
    cellsoup_ptr = cellsoup_ptr.find_previous("br")
    return cellsoup_ptr.find_next("a")["href"]

def extract_gbl_ausz(cellsoup: BeautifulSoup) -> str:
    return cellsoup.findAll("a")[1]["href"]

