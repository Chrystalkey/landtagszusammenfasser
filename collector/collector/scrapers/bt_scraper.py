import logging
from typing import List, Dict, Any, Optional
from datetime import datetime, date, timedelta
import uuid
import aiohttp
import json
import re
import openapi_client.models as models
from collector.interface import Scraper
from collector.document import Document


logger = logging.getLogger(__name__)


class BundestagAPIScraper(Scraper):
    CURRENT_WP = 20
    BT_API_KEY = "I9FKdCn.hbfefNWCY336dL6x62vfwNKpoN2RZ1gp21"

    def __init__(self, config, session: aiohttp.ClientSession):
        listing_urls = ["https://search.dip.bundestag.de/api/v1"]
        super().__init__(config, uuid.uuid4(), listing_urls, session)
        # Add headers for API key authentication
        self.session.headers.update({
            'api-key': config.api_key
        })
        self.vorgaenge = {}  # Dictionary zum Speichern der Vorgangsdaten
        self.positionen = {} # Dictionary zum Speichern der Vorgangspositionen
    
    async def listing_page_extractor(self, url) -> list[str]:
        """
        Holt Gesetzesvorhaben von der Bundestags-API
        """
        tage = 5
        startdatum = datetime.today() - timedelta(days=tage)
        #startdatum = datetime(2025, 2, 3)
        startdatum = startdatum.strftime("%Y-%m-%dT00:00:00")
        enddatum = datetime.today()
        #enddatum = datetime(2025, 2, 3)
        enddatum = enddatum.strftime("%Y-%m-%dT23:59:59")
        endpoint = f"{self.listing_urls[0]}/vorgang"
        params = {
            "apikey": self.BT_API_KEY,
            "f.aktualisiert.start" : startdatum,
            "f.aktualisiert.end" : enddatum,
            "f.vorgangstyp" : "Gesetzgebung",
            "f.wahlperiode" : self.CURRENT_WP
        }
        
        async with self.session.get(endpoint, params=params) as response:
            if response.status == 200:
                data = await response.json()
                num = data['numFound']
                logger.info(f"{num} Vorgang gefunden für Zeitraum {startdatum} - {enddatum}")
                documents = data.get("documents", [])
                for doc in documents:
                    self.vorgaenge[str(doc.get("id"))] = doc
                return list(self.vorgaenge.keys())
            else:
                logger.debug(f"Error while fetching listing page: {await response.text()}")
                return []
        
    async def item_extractor(self, vorgang_id: str) -> models.Vorgang:
        """
        Transformiert die Bundestags-API-Daten in das Format der OpenAPI-Spezifikation
        """
        if vorgang_id not in self.vorgaenge:
            logger.error(f"Vorgang {vorgang_id} nicht in gespeicherten Daten gefunden")
            return None
            
        vorgang = self.vorgaenge[vorgang_id]
        positionen = await self._get_vorgangspositionen(vorgang_id)

        # Basis-Gesetzesvorhaben erstellen
        gsvh = models.Vorgang.from_dict({
            "api_id": str(uuid.uuid4()),
            "titel": vorgang.get("titel", ""),
            "verfassungsaendernd": "Änderung des Grundgesetzes" in vorgang.get("titel", ""),
            "trojaner": False,  
            "initiatoren": self._extract_initiatoren(vorgang), 
            "typ": self._get_vorgangstyp(vorgang),
            "wahlperiode": self.CURRENT_WP,  
            "ids": [
                models.VgIdent.from_dict({
                    "typ": "vorgnr",
                    "id": str(vorgang.get("id"))
                }),
                models.VgIdent.from_dict({
                    "typ": "initdrucks",
                    "id": self._get_initdrucks_nummer(positionen)
                })
            ],
            "links": [self._create_dip_url(vorgang.get("id"), vorgang.get("titel"))], 
            "stationen": self._extract_stationen(positionen) #ToDo: Checken und mappen aus beratungsstand
        })
        
        logger.info(f"Daten: {gsvh}")
        return gsvh

    async def _get_vorgangspositionen(self, vorgang_id: str) -> List[Dict]:
        """
        Holt die Vorgangspositionen zum Vorgang
        """
        endpoint = f"{self.listing_urls[0]}/vorgangsposition"
        params = {
            "apikey": self.BT_API_KEY,
            "f.vorgang" : vorgang_id,
            "f.wahlperiode" : self.CURRENT_WP
        }

        async with self.session.get(endpoint, params=params) as response:
            if response.status == 200:
                data = await response.json()
                position = data.get("documents", [0])
                return position
            else:
                logger.debug(f"Error while fetching vorgangspositionen: {await response.text()}")
                return []

    def _get_initdrucks_nummer(self, positionen: List[Dict]) -> str:
        """
        Extrahiert die Dokumentennummer aus der Fundstelle für Vorgangspositionen vom Typ 'Gesetzentwurf'
        
        """
        for pos in positionen:
            if pos.get("vorgangsposition") == "Gesetzentwurf":
                return pos.get("fundstelle", {}).get("dokumentnummer", "")
        return ""

    def _get_vorgangstyp(self, vorgang: Dict) -> str:
        """Erkennung Zustimmung/Einspruchsgesetz"""
        zustimmungen = vorgang.get("zustimmungsbeduerftigkeit", [])
        
        if not zustimmungen:
            return "sonstig"
            
        gefundene_typen = set()
        
        for item in zustimmungen:
            if item.startswith("Nein"):
                gefundene_typen.add("gg-einspruch")
            elif item.startswith("Ja"):
                gefundene_typen.add("gg-zustimmung")
        
        # Wenn mehrere unterschiedliche Typen gefunden wurden oder kein Typ erkannt wurde
        if len(gefundene_typen) != 1:
            return "sonstig"
            
        # Ansonsten den einzigen gefundenen Typ zurückgeben
        return gefundene_typen.pop()
    
    def _extract_initiatoren(self, vorgang: Dict) -> List[str]:
        """Extrahiert die Initiatoren aus den Vorgangsdaten"""
        initiatoren = []
        if vorgang.get("initiative"):
            initiatoren.extend(vorgang["initiative"])
        return initiatoren

    def _extract_stationen(self, positionen: List[Dict]) -> List[models.Station]:
        """Extrahiert die Station aus den Vorgangsdaten"""
        stationen = []
        
        for position in positionen:
            station = self._create_station_from_position(position)
            if station:
                stationen.append(station)
        
        return stationen

    def _create_station_from_position(self, position: Dict) -> Optional[models.Station]:
        """Erstellt eine Station aus einer Vorgangsaktivität"""
        station_mapping = {
            "Gesetzentwurf": models.Stationstyp.PARL_MINUS_INITIATIV,
            "1. Beratung": models.Stationstyp.PARL_MINUS_AUSSCHBER,
            "Durchgang": models.Stationstyp.PARL_MINUS_AUSSCHBER,
            "1. Durchgang": models.Stationstyp.PARL_MINUS_AUSSCHBER,
            "Beschlussempfehlung und Bericht": models.Stationstyp.PARL_MINUS_BERABGESCHL,
            "Beschlussempfehlung": models.Stationstyp.PARL_MINUS_BERABGESCHL,
            "Empfehlungen der Ausschüsse": models.Stationstyp.PARL_MINUS_BERABGESCHL,
            "Bericht gemäß § 96 Geschäftsordnung BT": models.Stationstyp.PARL_MINUS_VERZOEGERT,
            "2. Beratung": models.Stationstyp.PARL_MINUS_AUSSCHBER,
            "3. Beratung": models.Stationstyp.PARL_MINUS_AUSSCHBER,
            "2. Durchgang": models.Stationstyp.PARL_MINUS_AUSSCHBER,
            
            
        }
        
        typ = station_mapping.get(position.get("vorgangsposition"))
        if not typ:
            typ = models.Stationstyp.SONSTIG
            
        datum = self._parse_date(position.get("datum"))
        return models.Station.from_dict({
            "datum": datum,
            "start_zeitpunkt": f"{datum}T00:00:00",  # Startzeitpunkt als Datum mit 00:00:00
            "dokumente": [],  # Leere Liste als Standardwert, ToDo: siehe Notizen
            "link": position.get("fundstelle").get("pdf_url"),
            "parlament": position.get("zuordnung"),
            "typ": typ,
        })

    def _extract_dokumente(self, drucksache: Dict) -> List[models.Dokument]:
        """Extrahiert Dokumente aus einer Drucksache"""
        if not drucksache:
            return []
            
        return [models.Dokument.from_dict({
            "titel": drucksache.get("titel", ""),
            "last_mod": datetime.now().isoformat(),
            "link": drucksache.get("url", ""),
            "hash": "",  # Muss noch implementiert werden
            "typ": models.Dokumententyp.DRUCKSACHE,
            "zusammenfassung": "",
            "schlagworte": [],
            "autorpersonen": [],
            "autoren": []
        })]

    def _create_dip_url(self, vorgangid, titel):
        #Bildet die URL zum Bundestags DIP aus dem Gesetzestitel
        cleantitle = re.sub(r"[^a-zA-Z0-9]", "-", titel)
        cleantitle = re.sub(r"--", "-", cleantitle)
        cleantitle = cleantitle.lower()
        cleantitle = cleantitle[:100]
        
        url = "https://dip.bundestag.de/vorgang/" + cleantitle + "/" + str(vorgangid)
        return url
    

    def _parse_date(self, date_str: str) -> str:
        """Konvertiert ein Datum-String in das erwartete ISO-Format"""
        if not date_str:
            return datetime.now().date().isoformat()
        try:
            return datetime.strptime(date_str, "%Y-%m-%d").date().isoformat()
        except ValueError:
            return datetime.now().date().isoformat()

