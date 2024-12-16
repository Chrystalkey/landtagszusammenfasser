from bs4 import BeautifulSoup
import requests
import openapi_client.models as models
from datetime import date
import uuid
import asyncio
import aiohttp
import logging

from collector.interface import Scraper

logger = logging.getLogger(__name__)
class BYLTScraper(Scraper):
    def __init__(self, config, session: aiohttp.ClientSession):
        CURRENT_WP = 19
        RESULT_COUNT = 200
        listing_urls = [
            f"https://www.bayern.landtag.de/parlament/dokumente/drucksachen?isInitialCheck=0&q=&dknr=&suchverhalten=AND&dokumentenart=Drucksache&ist_basisdokument=on&sort=date&anzahl_treffer={RESULT_COUNT}&wahlperiodeid%5B%5D={CURRENT_WP}&erfassungsdatum%5Bstart%5D=&erfassungsdatum%5Bend%5D=&dokumentenart=Drucksache&suchvorgangsarten%5B%5D=Gesetze%5C%5CGesetzentwurf&suchvorgangsarten%5B%5D=Gesetze%5C%5CStaatsvertrag&suchvorgangsarten%5B%5D=Gesetze%5C%5CHaushaltsgesetz%2C+Nachtragshaushaltsgesetz"
        ]
        super().__init__(config, uuid.uuid4(), listing_urls, session)

    async def listing_page_extractor(self, url) -> list[str]:
        # assumes a full page without pagination
        async with self.session.get(url) as get_result:
            soup = BeautifulSoup(await get_result.text(), "html.parser")
            # finds all result boxes
            resdiv_soup = soup.find_all("div", class_="row result")
            vgpage_urls = []
            for div in resdiv_soup:
                inner_div = div.find("div")
                heading = inner_div.find("h5")
                a_tags = inner_div.find_all("a", class_="link-with-icon")
                for a in a_tags:
                    if "views/vorgangsanzeige" in a["href"]:
                        vgpage_urls.append(str(a["href"]).strip())
            assert len(vgpage_urls) != 0, "Error: No Entry extracted from listing page"
            return vgpage_urls

    async def item_extractor(self, listing_item) -> models.Gesetzesvorhaben:
        global logger
        async with self.session.get(listing_item) as get_result:
            soup = BeautifulSoup(await get_result.text(), "html.parser")
            vorgangs_table = soup.find('tbody', id='vorgangsanzeigedokumente_data')
            rows = vorgangs_table.findAll("tr")

            btext_soup = soup.find("span", id="basistext")
            assert btext_soup != None, f"Error: Could not find Basistext for url {listing_item}"
            inds = btext_soup.text.split("Nr. ")[1].split(" vom")[0]
            logger.debug(f"GSVH mit Initiativdrucksache: {inds}")
            gsvh = models.Gesetzesvorhaben.from_dict({
                "api_id": str(uuid.uuid4()),
                "titel": soup.find("span", id="betreff").text,
                "verfassungsaendernd" : False,
                "initiatoren": [],
                "typ": models.Gesetzestyp.LANDGG,
                "ids" : [models.Identifikator.from_dict({"typ": "drucksnr", "id": str(inds)})],
                "links" : [listing_item],
                "stationen": []
            })

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
                stat = models.Station.from_dict({
                    "zeitpunkt": timestamp,
                    "gremium": "",
                    "dokumente": [],
                    "url": listing_item,
                    "parlament": "BY",
                    "schlagworte": [],
                    "stellungnahmen": [],
                    "typ": models.Stationstyp.POSTPARL_MINUS_KRAFT,
                    "trojaner": False,
                })
                cellclass = self.classify_object(cells[1])
                #print(f"Timestamp: {timestamp} Cellclass: {cellclass}")
                if cellclass == "initiativdrucksache":
                    link = extract_singlelink(cells[1])
                    gsvh.links.append(link)
                    stat.typ = models.Stationstyp.PARL_MINUS_INITIATIV
                    stat.gremium = "landtag"
                    stat.dokumente = [await self.create_document(link, "drucksache")]
                elif cellclass == "stellungnahme":
                    assert len(gsvh.stationen) > 0, "Error: Stellungnahme ohne Vorhergehenden Gesetzestext"
                    stln_urls = extract_schrstellung(cells[1])
                    stln = models.Stellungnahme.from_dict({
                        "meinung": 0,
                        "dokument": await self.create_document(stln_urls["stellungnahme"], "stellungnahme"),
                        "lobbyregister_url": stln_urls["lobbyregister"]
                    })
                    assert len(gsvh.stationen) > 0, "Error: Stellungnahme ohne Vorhergehenden Gesetzestext"
                    gsvh.stationen[-1].stellungnahmen.append(stln)
                    continue
                elif cellclass == "plenumsdiskussion-uebrw":
                    stat.typ = "parl-vollvlsgn"
                    stat.gremium = "landtag"
                    stat.dokumente = [await self.create_document(extract_plenproto(cells[1]), "protokoll")]
                elif cellclass == "plenumsdiskussion-zustm":
                    stat.typ = "parl-akzeptanz"
                    stat.gremium = "landtag"
                    stat.dokumente = [await self.create_document(extract_plenproto(cells[1]), "protokoll")]
                elif cellclass == "plenumsdiskussion-ablng":
                    stat.typ = "parl-ablehnung"
                    stat.gremium = "landtag"
                elif cellclass == "plenumsbeschluss":
                    stat.typ = "parl-schlussab"
                    stat.gremium = "landtag"
                    stat.dokumente = [await self.create_document(extract_singlelink(cells[1]), "drucksache")]
                elif cellclass == "ausschussbericht":
                    stat.typ = "parl-ausschber"
                    soup : BeautifulSoup = cells[1]
                    stat.gremium = soup.text.split("\n")[1]
                    stat.dokumente = [await self.create_document(extract_singlelink(cells[1]), "drucksache")]
                elif cellclass == "gesetzesblatt":
                    stat.gremium = "Gesetzesblatt"
                    stat.typ = "postparl-gsblt"
                    stat.dokumente = [await self.create_document(extract_singlelink(cells[1]), "sonstig")]
                elif cellclass == "unclassified":
                    logger.warning("Warning: Unclassified cell. Discarded.")
                
                stat.trojaner = detect_trojaner(stat)
                gsvh.stationen.append(stat)
            #print(len(rows))
            return gsvh

    async def create_document(self, url: str, type_hint : str) -> models.Dokument:
        global logger
        logger.debug(f"Creating document from url: {url}")
        document_info = await extract_pdf_drucks(url, self.session)
        autoren = []
        if document_info["author"] is not None:
            autoren.append(f"{document_info["author"]} (Author)")
        if document_info["creator"] is not None:
            autoren.append(f"{document_info["creator"]} (Creator)")


        if document_info["title"] is not None:
            titel = document_info["title"]
        elif document_info["subject"] is not None:
            titel = document_info["subject"]
        else:
            titel = "Unbekannt"

        dok_dic = {
            "zeitpunkt": date.today(),
            "titel": titel,
            "url": url,
            "hash": self.hash_pdf(url),
            "typ": type_hint,
            "zusammenfassung": "TODO",
            "autoren": autoren,
            "schlagworte": [],
        }
        return models.Dokument.from_dict(dok_dic)
    
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

def detect_trojaner(stat: models.Station) -> bool:
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

async def extract_pdf_drucks(url, session):
    import PyPDF2
    import os
    async with session.get(url) as pdfFile:
        
        global logger
        randid = str(uuid.uuid4())
        logger.debug(f"Extracting PDF Metadata for Url {url}, writing to file {randid}.pdf")
        with open(f"{randid}.pdf", "wb") as f:
            f.write(await pdfFile.read())
            pdfFileObj = open(f"{randid}.pdf", "rb")
            reader = PyPDF2.PdfReader(pdfFileObj)
            meta = reader.metadata
            dic= {
                "title": str(meta.title),
                "author": meta.author,
                "creator": meta.creator,
                "subject": meta.subject
            }
            pdfFileObj.close()
        if os.path.exists(f"{randid}.pdf"):
            os.remove(f"{randid}.pdf")
        return dic