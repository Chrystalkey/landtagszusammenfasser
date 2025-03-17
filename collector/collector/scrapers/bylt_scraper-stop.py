import hashlib
import logging
import os
import re
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

                # date is in the first cell. If its just an announcement, just skip it
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

                ### Initialize Station scaffold
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
                        "additional_links": [],
                    }
                )

                cellclass = self.classify_cell(cells[1])
                
                ## initiativ
                ## has: one doklink, drucksnr, new station
                if cellclass == "initiativ":
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
                    stat.betroffene_texte = list(set(dok.texte))
                elif cellclass == "unknown":
                    logger.warning(f"Unknown Cell class for VG {listing_item}\nContents: {cells[1].text}")
                    continue
                elif cellclass == "ignored":
                    continue
                ## stellungnahme
                ## is added to the exactly preceding station
                ## has: one doklink, name des/der stellungnehmenden (=autor)
                elif cellclass == "stellungnahme":
                    assert (
                        len(vg.stationen) > 0
                    ), "Error: Stellungnahme ohne Vorhergehenden Gesetzestext"
                    stln_urls = extract_schrstellung(cells[1])
                    dok = await self.create_document(stln_urls["stellungnahme"], models.Doktyp.STELLUNGNAHME)
                    if stln_urls["autor"]:
                        if dok.authoren is None:
                            dok.authoren = stln_urls["autor"]
                        else:
                            dok.authoren.append(stln_urls["autor"])
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
                ## Zelle mit Plenarprotokoll
                ## has: link(plenarprotokoll), link(Auszug-plenarprotokoll), link(Videoausschnitt)
                ## neue station oder merge
                elif cellclass.startswith("plenum-proto"):
                    pproto = extract_plenproto(cells[1])
                    gremium = models.Gremium.from_dict({"name": "plenum", "parlament": "BY","wahlperiode": 19})
                    dok = await self.create_document(pproto["pprotoaz"], models.Doktyp.PLENAR_MINUS_PROTOKOLL)
                    betroffene_texte = list(set(dok.texte))
                    typ = None
                    video_link = pproto.get("video")
                    if cellclass == "plenum-proto-uebrw":
                        typ = "parl-vollvlsgn"
                    elif cellclass == "plenum-proto-zustm":
                        typ = "parl-akzeptanz"
                    elif cellclass == "plenum-proto-ablng":
                        typ = "parl-ablehnung"
                    elif cellclass == "plenum-proto-rueckzug":
                        typ = "parl-zurueckgz"
                    if len(vg.stationen) > 0 and vg.stationen[-1].typ == typ:
                        vg.stationen[-1].typ = typ
                        vg.stationen[-1].dokumente.append(models.DokRef(dok.package()))
                        vg.stationen[-1].gremium = gremium
                        vg.stationen[-1].betroffene_texte = betroffene_texte
                        vg.stationen[-1].additional_links.append(video_link)
                        continue
                    else:
                        stat.typ = typ
                        stat.dokumente = [models.DokRef(dok.package())]
                        stat.gremium = gremium
                        stat.betroffene_texte = betroffene_texte
                        stat.additional_links.append(video_link)

                ## Rückzugsmitteilung
                ## Ein Link
                elif cellclass == "rueckzug":
                    dok = await self.create_document(extract_singlelink(cells[1]), models.Doktyp.MITTEILUNG)
                    dok.drucksnr = extract_drucksnr(cells[1])
                    typ = models.Stationstyp.PARL_MINUS_ZURUECKGZ
                    betroffene_texte = list(set(dok.texte))
                    gremium = models.Gremium.from_dict({"name": "plenum", "parlament": "BY","wahlperiode": 19})
                    if len(vg.stationen) > 0 and vg.stationen[-1].typ == typ:
                        vg.stationen[-1].typ = typ
                        vg.stationen[-1].dokumente.append(models.DokRef(dok.package()))
                        vg.stationen[-1].gremium = gremium
                        vg.stationen[-1].betroffene_texte = betroffene_texte
                        continue
                    else:
                        stat.typ = typ
                        stat.dokumente = [models.DokRef(dok.package())]
                        stat.gremium = gremium
                        stat.betroffene_texte = betroffene_texte

                ## Plenumsentscheidung
                ## hat einen Dokumentenlink
                elif cellclass.startswith("plenum-beschluss"):
                    dok = await self.create_document(extract_singlelink(cells[1]), models.Doktyp.ENTWURF)
                    dok.drucksnr = extract_drucksnr(cells[1])
                    typ = None
                    trojanergefahr = max(dok.trojanergefahr, 1)
                    gremium = models.Gremium.from_dict({"name": "plenum", "parlament": "BY","wahlperiode": 19})
                    betroffene_texte = list(set(dok.texte))
                    if cellclass.endswith("zustm"):
                        typ = "parl-akzeptanz"
                    elif cellclass.endswith("ablng"):
                        typ = "parl-ablehnung"
                    if len(vg.stationen) > 0 and vg.stationen[-1].typ ==  typ:
                        vg.stationen[-1].typ = typ
                        vg.stationen[-1].dokumente.append(models.DokRef(dok.package()))
                        vg.stationen[-1].gremium = gremium
                        vg.stationen[-1].betroffene_texte = betroffene_texte
                        vg.stationen[-1].trojanergefahr = trojanergefahr
                        continue
                    else:
                        stat.typ = typ      
                        stat.dokumente = [models.DokRef(dok.package())]
                        stat.gremium = gremium
                        stat.betroffene_texte = betroffene_texte
                        stat.trojanergefahr = trojanergefahr
                ## Ausschussberichterstattung
                ## hat 1 Link: Beschlussempfehlung
                ## doppelt sich manchmal aus unbekannten Gründen
                elif cellclass == "ausschuss-bse":
                    dok = await self.create_document(extract_singlelink(cells[1]), models.Doktyp.BESCHLUSSEMPF)
                    dok.drucksnr = extract_drucksnr(cells[1])
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
                            existing_station.betroffene_texte = list(set(existing_station.betroffene_texte))
                        
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
                        stat.betroffene_texte = list(set(dok.texte))
                ## Gesetzblatt. Zwei Links, einer davon 
                elif cellclass == "gsblatt":
                    stat.gremium = models.Gremium.from_dict({
                        "name": "gesetzesblatt",
                        "parlament": "BY","wahlperiode": 19
                    })
                    stat.typ = "postparl-gsblt"
                    dok = await self.create_document(extract_singlelink(cells[1]), models.Doktyp.SONSTIG)
                    stat.dokumente = [models.DokRef(dok.package())]
                else:
                    logger.error(f"Reached an unreachable state with cellclass: {cellclass}. Discarded.")
                    continue
                stat.dokumente = dedup_drucks(stat.dokumente)
                logger.debug(
                    f"Adding New Station of class `{""+stat.typ}` to Vorgang `{vg.api_id}`"
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

    """Cellclasses:
    - initiativ                 # has Gesetzentwurf(Drucksache)
    - stellungnahme             # has Stellungnahme
    - ausschuss-bse             # has Beschlussempf(Drucksache)
    - plenum-proto-uebrw        # has Plenarprotokoll(Protokoll), Link zu Videoausschnitt
    - plenum-proto-zustm        # has Plenarprotokoll(Protokoll), Link zu Videoausschnitt
    - plenum-proto-ablng        # has Plenarprotokoll(Protokoll), Link zu Videoausschnitt
    - plenum-proto-keineentsch  # has Plenarprotokoll(Protokoll), Link zu Videoausschnitt
    - plenum-beschluss-zustm    # has Beschluss(Drucksache)
    - plenum-beschluss-ablng    # has Beschluss(Drucksache)
    - rueckzug                  # has Mitteilung(Drucksache)
    - gsblatt                   # has GVBL-Auszug(Gesetzblatt)
    - ignored                   # beratung oder ergebnis-folgt-dinge
    - unknown                   # unknown cell type
    Links die In summe alle typen enthalten:
    # https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157296
    # https://www.bayern.landtag.de/webangebot3/views/vorgangsanzeige/vorgangsanzeige.xhtml?gegenstandid=157725
    """
    def classify_cell(self, context: BeautifulSoup) -> str:
        cellsoup = context
        if cellsoup.text.find("Initiativdrucksache") != -1:       
            return "initiativ"
        elif (                                                    # 
            cellsoup.text.find("Schriftliche Stellungnahmen im Gesetzgebungsverfahren")
            != -1
        ):
            return "stellungnahme"
        elif (
            cellsoup.text.find("Plenum") != -1):
            if cellsoup.text.find("Plenarprotokoll") != -1:
                if cellsoup.text.find("Überweisung") != -1:
                    return "plenum-proto-uebrw"
                elif cellsoup.text.find("Zustimmung") != -1:
                    return "plenum-proto-zustm"
                elif cellsoup.text.find("Ablehnung") != -1:
                    return "plenum-proto-ablng"
                elif cellsoup.text.find("Rücknahme") != -1:
                    return "plenum-proto-rueckzug"
            else: # plenum aber kein plenarprotokoll == beschluss
                if cellsoup.text.find("Ablehnung") != -1:
                    return "plenum-beschluss-ablng"
                elif cellsoup.text.find("Zustimmung") != -1:
                    return "plenum-beschluss-zustm"
                elif cellsoup.text.find("Rücknahme") != -1:
                    return "rueckzug"
        elif cellsoup.text.find("Ausschuss") != -1:
            return "ausschuss-bse"
        elif cellsoup.text.find("Gesetz- und Verordnungsblatt") != -1:
            return "gsblatt"
        return "unknown"

def dedup_drucks(doks: list[models.DokRef]) -> list[models.Dokument]:
    unique_doks = []
    for d in doks:
        if d.actual_instance.drucksnr:
            found = False
            for e in unique_doks:
                if e.actual_instance.drucksnr and e.actual_instance.drucksnr == d.actual_instance.drucksnr:
                    found = True
                    break
            if found:
                continue
        unique_doks.append(d)
    return unique_doks

def extract_drucksnr(cellsoup: BeautifulSoup) -> str:
    match = None
    soupsplit = cellsoup.text.replace("\n", " ").split(" ")
    for slice in soupsplit:
        match = re.match(r"\d\d+\/\d+", slice)
        if match is not None:
            return match[0]
    if match is None:
        logger.error(f"Error: Expected to find DrucksNr in Cellsoup {cellsoup.text}")
    raise Exception(f"expected to extract drucksnr from {cellsoup.text}")

def extract_singlelink(cellsoup: BeautifulSoup) -> str:
    return cellsoup.find("a")["href"]

# returns: {"typ": link, ...}
def extract_schrstellung(cellsoup: BeautifulSoup) -> dict:
    links = cellsoup.find_all("a")
    assert (
        len(links) > 0 and len(links) < 3
    ), f"Error: Unexpected number of links in Stellungnahme: {len(links)}, in cellsoup `{cellsoup}`"
    if len(links) == 2:
        return {"lobbyregister": links[0]["href"], "stellungnahme": links[1]["href"], "autor": links[0].text if links[0].text != "Download PDF" else None}
    elif len(links) == 1:
        return {"stellungnahme": links[0]["href"], "lobbyregister": "", "autor": links[0].text if links[0].text != "Download PDF" else None}


def extract_plenproto(cellsoup: BeautifulSoup) -> str:
    cellsoup_ptr = cellsoup.find(string="Protokollauszug")
    cellsoup_ptr = cellsoup_ptr.find_previous("br")
    proto_link = cellsoup_ptr.find_next("a")["href"]
    video_link = None
    cellsoup_ptr = cellsoup.find_all("a")
    for link in cellsoup_ptr:
        if link.text == "Video zum TOP":
            video_link = link["href"]
    return {"pprotoaz": proto_link, "video": video_link}


def extract_gbl_ausz(cellsoup: BeautifulSoup) -> str:
    return cellsoup.findAll("a")[1]["href"]