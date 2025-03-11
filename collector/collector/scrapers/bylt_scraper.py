import hashlib
import logging
import os
import uuid
import datetime  # required because of the eval() call later down the line
from datetime import date as dt_date
from datetime import datetime as dt_datetime

import aiohttp
from bs4 import BeautifulSoup

import openapi_client.models as models
from collector.interface import Scraper
from collector.document import Document

logger = logging.getLogger(__name__)
NULL_UUID = uuid.UUID("00000000-0000-0000-0000-000000000000")
TEST_DATE = dt_datetime.fromisoformat("1940-01-01T00:00:00+00:00")

class BYLTScraper(Scraper):
    def __init__(self, config, session: aiohttp.ClientSession):
        CURRENT_WP = 19
        RESULT_COUNT = 200
        listing_urls = [
            f"https://www.bayern.landtag.de/parlament/dokumente/drucksachen?isInitialCheck=0&q=&dknr=&suchverhalten=AND&dokumentenart=Drucksache&ist_basisdokument=on&sort=date&anzahl_treffer={RESULT_COUNT}&wahlperiodeid%5B%5D={CURRENT_WP}&erfassungsdatum%5Bstart%5D=&erfassungsdatum%5Bend%5D=&dokumentenart=Drucksache&suchvorgangsarten%5B%5D=Gesetze%5C%5CGesetzentwurf&suchvorgangsarten%5B%5D=Gesetze%5C%5CStaatsvertrag&suchvorgangsarten%5B%5D=Gesetze%5C%5CHaushaltsgesetz%2C+Nachtragshaushaltsgesetz"
        ]
        super().__init__(config, uuid.uuid4(), listing_urls, session)
        # Add headers for API key authentication
        self.session.headers.update({
            'api-key': config.api_key
        })

    async def listing_page_extractor(self, url) -> list[str]:
        global logger
        # assumes a full page without pagination
        logger.debug(f"Extracting Listing Page `{url}`")
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

    async def item_extractor(self, listing_item) -> models.Vorgang:
        global logger, NULL_UUID, TEST_DATE
        async with self.session.get(listing_item) as get_result:
            soup = BeautifulSoup(await get_result.text(), "html.parser")
            vorgangs_table = soup.find("tbody", id="vorgangsanzeigedokumente_data")
            rows = vorgangs_table.find_all("tr")

            btext_soup = soup.find("span", id="basistext")
            assert (
                btext_soup != None
            ), f"Error: Could not find Basistext for url {listing_item}"
            inds = btext_soup.text.split("Nr. ")[1].split(" vom")[0]
            titel = soup.find("span", id="betreff") \
                    .text.replace("\n", " ") \
                    .replace("\r\n", " ")\
                    .replace("\r", " ")\
                    .strip()
            vg = models.Vorgang.from_dict(
                {
                    "api_id": str(uuid.uuid4()) if not self.config.testing_mode else str(NULL_UUID),
                    "titel": titel,
                    "kurztitel": titel,
                    "wahlperiode":  19,
                    "verfassungsaendernd": False,
                    "trojaner": False,
                    "initiatoren": [],
                    "typ": "gg-land-parl",
                    "ids": [
                        models.VgIdent.from_dict(
                            {"typ": "initdrucks", "id": str(inds)}
                        )
                    ],
                    "links": [listing_item],
                    "stationen": [],
                }
            )
            logger.debug(
                f"New vg mit Initiativdrucksache: {inds}, ApiID: {vg.api_id}"
            )

            # Initiatoren
            init_ptr = soup.find(string="Initiatoren")
            initiat_lis = init_ptr.find_next("ul").find_all("li")
            init_dings = []
            init_persn = []
            for ini in initiat_lis:
                if "(" not in ini.text:
                    init_dings.append(ini.text)
                else:
                    init_persn.append(ini.text)
            vg.initiatoren = init_dings
            if len(init_persn) > 0:
                vg.initiator_personen = init_persn
            assert (
                len(vg.initiatoren) > 0
            ), f"Error: Could not find Initiatoren for url {listing_item}"

            # Helper function to check if a station is a plenary session
            def is_plenary_session(station_typ):
                return station_typ in ["parl-initiativ", "parl-vollvlsgn", "parl-akzeptanz", "parl-ablehnung"]
            
            # Helper function to find a matching committee station
            def find_matching_committee_station(committee_name):
                # Iterate backward through stations to find the most recent committee with the same name
                # Only consider stations after the last plenary session
                for i in range(len(vg.stationen) - 1, -1, -1):
                    station = vg.stationen[i]
                    
                    # If we hit a plenary session, stop looking
                    if is_plenary_session(station.typ):
                        break
                        
                    # If we find a committee station with the same name, return it
                    if (station.typ == "parl-ausschber" and 
                        station.gremium.name == committee_name):
                        return i
                
                # No matching station found
                return -1

            # station extraction
            for row in rows:
                cells = row.find_all("td")

                assert (
                    len(cells) == 2
                ), f"Warning: Unexpectedly found more or less than exactly two gridcells in: `{row}` of url `{listing_item}`"

                # date is in the first cell
                if cells[0].text == "Beratung / Ergebnis folgt":
                    continue
                timestamp = cells[0].text.split(".")
                assert (
                    len(timestamp) == 3
                ), f"Error: Unexpected date format: `{timestamp}` of url `{listing_item}`"
                timestamp = dt_datetime(
                    year=int(timestamp[2]), month=int(timestamp[1]), day=int(timestamp[0]),
                    hour=0, minute=0, second=0
                ).astimezone(datetime.timezone.utc)
                # content is in the second cell
                if self.config.testing_mode:
                    timestamp = TEST_DATE
                stat = models.Station.from_dict(
                    {
                        "start_zeitpunkt": timestamp,
                        "dokumente": [],
                        "link": listing_item,
                        "parlament": "BY",
                        "schlagworte": [],
                        "stellungnahmen": [],
                        "typ": "postparl-kraft",
                        "trojaner": False,
                        "betroffene_texte": [],
                    }
                )
                cellclass = self.classify_cell(cells[1])
                # print(f"Timestamp: {timestamp} Cellclass: {cellclass}")
                if cellclass == "initiativdrucksache":
                    link = extract_singlelink(cells[1])
                    vg.links.append(link)
                    stat.typ = "parl-initiativ"
                    stat.gremium = models.Gremium.from_dict({
                        "name": "plenum",
                        "parlament": "BY",
                        "wahlperiode": 19
                    })

                    dok = await self.create_document(link, models.Doktyp.ENTWURF)
                    dok.drucksnr = str(inds)
                    stat.dokumente = [models.DokRef(dok.package())]
                    stat.trojanergefahr = max(dok.trojanergefahr, 1)
                    stat.betroffene_texte = dok.texte
                elif cellclass == "stellungnahme":
                    assert (
                        len(vg.stationen) > 0
                    ), "Error: Stellungnahme ohne Vorhergehenden Gesetzestext"
                    stln_urls = extract_schrstellung(cells[1])
                    dok = await self.create_document(stln_urls["stellungnahme"], models.Doktyp.STELLUNGNAHME)
                    stln = models.Stellungnahme.from_dict(
                        {
                            "meinung": max(dok.meinung or 1, 1),
                            "dokument": dok.package(),
                            "lobbyregister_url": stln_urls["lobbyregister"],
                        }
                    )
                    assert (
                        len(vg.stationen) > 0
                    ), "Error: Stellungnahme ohne Vorhergehenden Gesetzestext"
                    vg.stationen[-1].stellungnahmen.append(stln)
                    continue
                elif cellclass == "plenumsdiskussion-uebrw":
                    stat.typ = "parl-vollvlsgn"
                    stat.gremium = models.Gremium.from_dict({"name": "plenum", "parlament": "BY","wahlperiode": 19})
                    dok = await self.create_document(extract_plenproto(cells[1]), models.Doktyp.PLENAR_MINUS_PROTOKOLL)
                    stat.betroffene_texte = dok.texte
                    stat.dokumente = [models.DokRef(dok.package())]
                elif cellclass == "plenumsdiskussion-zustm":
                    dok = await self.create_document(extract_plenproto(cells[1]), models.Doktyp.PLENAR_MINUS_PROTOKOLL)
                    
                    if len(vg.stationen) > 0 and vg.stationen[-1].typ == "parl-akzeptanz":
                        vg.stationen[-1].dokumente.append(models.DokRef(dok.package()))
                        continue
                    else:
                        stat.typ = "parl-akzeptanz"
                        stat.gremium = models.Gremium.from_dict({"name": "plenum", "parlament": "BY","wahlperiode": 19})
                        
                        stat.dokumente = [models.DokRef(dok.package())]
                elif cellclass == "rueckzugmeldung":
                    dok = await self.create_document(extract_singlelink(cells[1]), models.Doktyp.MITTEILUNG)
                    stat.typ = models.Stationstyp.PARL_MINUS_ZURUECKGZ
                    stat.trojanergefahr = max(dok.trojanergefahr, 1)
                    stat.betroffene_texte = dok.texte
                    stat.gremium = models.Gremium.from_dict({"name": "plenum", "parlament": "BY","wahlperiode": 19})
                    stat.dokumente = [models.DokRef(dok.package())]
                    
                elif cellclass == "plenumsmitteilung-rueckzug":
                    if (len(vg.stationen) > 0 and vg.stationen[-1].typ == "parl-zurueckgz"):
                        dok = await self.create_document(extract_plenproto(cells[1]), models.Doktyp.PLENAR_MINUS_PROTOKOLL)
                        vg.stationen[-1].typ = "parl-zurueckgz"
                        vg.stationen[-1].dokumente.append(models.DokRef(dok.package()))
                    else:
                        logger.warning(f"Warnung: Plenumsmitteilung über Gesetzesrücknahme ohne vorherige Zeile über Rücknahme")
                    continue
                elif cellclass == "plenumsdiskussion-ablng":
                    if len(vg.stationen) > 0 and vg.stationen[-1].typ in ["parl-akzeptanz", "parl-ablehnung"]:
                        vg.stationen[-1].typ = "parl-ablehnung"
                        continue
                    else:
                        stat.typ = "parl-ablehnung"
                        stat.gremium = models.Gremium.from_dict({
                            "name": "plenum", 
                            "parlament": "BY","wahlperiode": 19
                        })
                elif cellclass == "plenumsbeschluss":
                    dok = await self.create_document(extract_singlelink(cells[1]), models.Doktyp.ENTWURF)
                    if len(vg.stationen) > 0 and vg.stationen[-1].typ in ["parl-akzeptanz", "parl-ablehnung"]:
                        vg.stationen[-1].dokumente.append(models.DokRef(dok.package()))
                        vg.stationen[-1].trojanergefahr = max(dok.trojanergefahr, 1)
                        vg.stationen[-1].betroffene_texte = dok.texte
                        continue
                    else:
                        stat.typ = models.Stationstyp.PARL_MINUS_ABLEHNUNG
                        stat.gremium = models.Gremium.from_dict({
                            "name": "plenum", 
                            "parlament": "BY","wahlperiode": 19
                        })
                        stat.trojanergefahr = max(dok.trojanergefahr, 1)
                        stat.betroffene_texte = dok.texte
                        stat.dokumente = [models.DokRef(dok.package())]
                elif cellclass == "ausschussbericht":
                    dok = await self.create_document(extract_singlelink(cells[1]), models.Doktyp.BESCHLUSSEMPF)
                    soup: BeautifulSoup = cells[1]
                    ausschuss_name = soup.text.split("\n")[1]
                    
                    # Check if there's an existing committee station to merge with
                    existing_idx = find_matching_committee_station(ausschuss_name)
                    
                    if existing_idx >= 0:
                        # Merge with existing committee station
                        logger.debug(f"Merging ausschussbericht for committee '{ausschuss_name}'")
                        existing_station = vg.stationen[existing_idx]
                        existing_station.dokumente.append(dok.package())
                        
                        # Update trojaner flag if necessary
                        existing_station.trojanergefahr = max(dok.trojanergefahr, 1)
                        
                        # Merge betroffene_texte lists
                        if dok.texte:
                            if not existing_station.betroffene_texte:
                                existing_station.betroffene_texte = []
                            existing_station.betroffene_texte.extend(dok.texte)
                        
                        continue
                    else:
                        # Create new committee station
                        stat.typ = "parl-ausschber"
                        stat.gremium = models.Gremium.from_dict({
                            "name": ausschuss_name,
                            "parlament": "BY","wahlperiode": 19
                        })
                        stat.dokumente = [models.DokRef(dok.package())]
                        stat.trojanergefahr = max(dok.trojanergefahr, 1)
                        stat.betroffene_texte = dok.texte

                elif cellclass == "gesetzesblatt":
                    stat.gremium = models.Gremium.from_dict({
                        "name": "Gesetzesblatt",
                        "parlament": "BY","wahlperiode": 19
                    })
                    stat.typ = "postparl-gsblt"
                    dok = await self.create_document(extract_singlelink(cells[1]), models.Doktyp.SONSTIG)
                    stat.dokumente = [models.DokRef(dok.package())]
                elif cellclass == "unclassified":
                    logger.warning("Warning: Unclassified cell. Discarded.")
                    continue
                else:
                    logger.error("Reached an unreachable state. Discarded.")
                    continue
                logger.debug(
                    f"Adding New Station of class `{""+stat.typ}` to GSVH `{vg.api_id}`"
                )
                vg.stationen.append(stat)
            return vg

    async def create_document(self, url: str, type_hint: models.Doktyp) -> Document:
        global logger
        logger.debug(f"Creating document from url: {url}")
        document = self.config.cache.get_dokument(url)
        if document is None:
            logger.debug("Cached version not found, fetching from source")
            document = Document(self.session, url, type_hint, self.config)
            await document.run_extraction()
            self.config.cache.store_dokument(url, document)
            return document
        else:
            logger.debug("Cached version found, used")
            return self.config.cache.get_dokument(url)

    def classify_cell(self, context: BeautifulSoup) -> str:
        cellsoup = context
        if cellsoup.text.find("Initiativdrucksache") != -1:
            return "initiativdrucksache"
        elif (
            cellsoup.text.find("Schriftliche Stellungnahmen im Gesetzgebungsverfahren")
            != -1
        ):
            return "stellungnahme"
        elif cellsoup.text.find("Plenum") != -1 and \
            cellsoup.text.find("Rücknahme") != -1 and \
            cellsoup.text.find("Plenarprotokoll") == -1:
            return "rueckzugmeldung"
        elif (
            cellsoup.text.find("Plenum") != -1
            and cellsoup.text.find("Plenarprotokoll") != -1
        ):
            if cellsoup.text.find("Überweisung") != -1:
                return "plenumsdiskussion-uebrw"
            elif cellsoup.text.find("Zustimmung") != -1:
                return "plenumsdiskussion-zustm"
            elif cellsoup.text.find("Ablehnung") != -1:
                return "plenumsdiskussion-ablng"
            elif cellsoup.text.find("Plenarprotokoll") != -1 and cellsoup.text.find("Rücknahme") != -1:
                return "plenumsmitteilung-rueckzug"
            else:
                print(
                    f"Warning: Plenumsdiskussion without specific classification: `{cellsoup}`"
                )
                return "unclassified"
        elif cellsoup.text.find("Ausschuss") != -1:
            return "ausschussbericht"
        elif cellsoup.text.find("Gesetz- und Verordnungsblatt") != -1:
            return "gesetzesblatt"
        else:
            return "unclassified"

def extract_singlelink(cellsoup: BeautifulSoup) -> str:
    return cellsoup.find("a")["href"]

# returns: {"typ": link, ...}
def extract_schrstellung(cellsoup: BeautifulSoup) -> dict:
    links = cellsoup.findAll("a")
    assert (
        len(links) > 0 and len(links) < 3
    ), f"Error: Unexpected number of links in Stellungnahme: {len(links)}, in cellsoup `{cellsoup}`"
    if len(links) == 2:
        return {"lobbyregister": links[0]["href"], "stellungnahme": links[1]["href"]}
    elif len(links) == 1:
        return {"stellungnahme": links[0]["href"], "lobbyregister": ""}


def extract_plenproto(cellsoup: BeautifulSoup) -> str:
    cellsoup_ptr = cellsoup.find(string="Protokollauszug")
    cellsoup_ptr = cellsoup_ptr.find_previous("br")
    return cellsoup_ptr.find_next("a")["href"]


def extract_gbl_ausz(cellsoup: BeautifulSoup) -> str:
    return cellsoup.findAll("a")[1]["href"]