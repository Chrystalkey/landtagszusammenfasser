import logging
from typing import List, Dict, Any, Optional
from datetime import datetime, date
import uuid
import aiohttp
import json
import openapi_client.models as models
from collector.interface import Scraper
from collector.document import Document


logger = logging.getLogger(__name__)


class BundestagAPIScraper(Scraper):
    def __init__(self, config, session: aiohttp.ClientSession):
        listing_urls = ["https://search.dip.bundestag.de/api/v1"]
        super().__init__(config, uuid.uuid4(), listing_urls, session)
        # Add headers for API key authentication
        self.session.headers.update({
            'api-key': config.api_key
        })
        self.vorgaenge = {}  # Dictionary zum Speichern der Vorgangsdaten
    
    async def listing_page_extractor(self, url) -> list[str]:
        """
        Holt Gesetzesvorhaben von der Bundestags-API
        """
        CURRENT_WP = 20
        tage = 30
        bt_api_key = "I9FKdCn.hbfefNWCY336dL6x62vfwNKpoN2RZ1gp21"
        #startdatum = datetime.today() - datetime.timedelta(days=tage)
        startdatum = datetime(2025, 2, 3)
        startdatum = startdatum.strftime("%Y-%m-%dT00:00:00")
        #enddatum = datetime.today()
        enddatum = datetime(2025, 2, 3)
        enddatum = enddatum.strftime("%Y-%m-%dT23:59:59")
        endpoint = f"{self.listing_urls[0]}/vorgang"
        params = {
            "apikey": bt_api_key,
            "f.aktualisiert.start" : startdatum,
            "f.aktualisiert.end" : enddatum,
            "f.vorgangstyp" : "Gesetzgebung",
            "f.wahlperiode" : CURRENT_WP
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
        
    async def item_extractor(self, vorgang_id: str) -> models.Gesetzesvorhaben:
        """
        Transformiert die Bundestags-API-Daten in das Format der OpenAPI-Spezifikation
        """
        if vorgang_id not in self.vorgaenge:
            logger.error(f"Vorgang {vorgang_id} nicht in gespeicherten Daten gefunden")
            return None
            
        vorgang = self.vorgaenge[vorgang_id]
        # Basis-Gesetzesvorhaben erstellen
        gsvh = models.Gesetzesvorhaben.from_dict({
            "api_id": str(uuid.uuid4()),
            "titel": vorgang.get("titel", ""),
            #ToDo: Kurztitel/Offizieller Titel
            "parlament": "BT", #ToDo: Abhängig von Zuordnung vorgangsposition
            "verfassungsaendernd": "Grundgesetz" in vorgang.get("titel", ""), #ToDo: FUnktioniert das?
            "trojaner": False,  
            "initiatoren": self.extract_initiatoren(vorgang), #ToDo: Checken / Was ist mit den Ministerien?
            "typ": "bt-parlament",  #ToDo: Erkennung Zustimmung/Einspruchsgesetz
            "wahlperiode": vorgang.get("wahlperiode", 20),  # Aktuelle Wahlperiode als Standard
            "ids": [
                models.Identifikator.from_dict({
                    "typ": "vorgnr",
                    "id": str(vorgang.get("id"))  #ToDo: initdrucks zusätzlich aus vorgangsposition.
                })
            ],
            "links": [f"https://pdok.bundestag.de/vorgang/{vorgang.get('id')}"],  #ToDo: Korrekten Link generieren
            "stationen": self.extract_stationen(vorgang) #ToDo: Checken und mappen aus beratungsstand
        })
        
        logger.info(f"Daten: {gsvh}")
        return gsvh

    def extract_initiatoren(self, vorgang: Dict) -> List[str]:
        """Extrahiert die Initiatoren aus den Vorgangsdaten"""
        initiatoren = []
        if vorgang.get("initiative"):
            initiatoren.extend(vorgang["initiative"])
        return initiatoren

    def extract_stationen(self, vorgang: Dict) -> List[models.Station]:
        """Extrahiert die Stationen aus den Vorgangsdaten"""
        stationen = []
        
        # Beispiel für eine Initiativ-Station
        if vorgang.get("initiative"):
            init_station = models.Station.from_dict({
                "datum": self._parse_date(vorgang.get("initiativDatum")),
                "gremium": "Bundestag",
                "dokumente": self._extract_dokumente(vorgang.get("initiativDrucksache")),
                "link": f"https://pdok.bundestag.de/vorgang/{vorgang.get('vorgangsId')}",
                "parlament": "BT",
                "schlagworte": vorgang.get("sachgebiet", []),
                "stellungnahmen": [],
                "typ": models.Stationstyp.PARL_MINUS_INITIATIV,
                "trojaner": False,
                "betroffene_texte": []
            })
            stationen.append(init_station)
        
        # Weitere Stationen aus Vorgangsablauf extrahieren
        for aktivitaet in vorgang.get("vorgangsablauf", []):
            station = self._create_station_from_aktivitaet(aktivitaet)
            if station:
                stationen.append(station)
        
        return stationen

    def _create_station_from_aktivitaet(self, aktivitaet: Dict) -> Optional[models.Station]:
        """Erstellt eine Station aus einer Vorgangsaktivität"""
        station_mapping = {
            "Dem Bundestag zugeleitet - Noch nicht beraten": models.Stationstyp.PARL_MINUS_INITIATIV,
            "Dem Bundesrat zugeleitet - Noch nicht beraten": models.Stationstyp.PARL_MINUS_INITIATIV,
            "Noch nicht beraten": models.Stationstyp.PARL_MINUS_INITIATIV,
            "Überwiesen": models.Stationstyp.PARL_MINUS_AUSSCHBER,
            "Im Vermittlungsverfahren": models.Stationstyp.PARL_MINUS_AUSSCHBER,
            "Zustimmung": models.Stationstyp.PARL_MINUS_AKZEPTANZ,
            "Verkündet": models.Stationstyp.POSTPARL_MINUS_GSBLT,
            "Für erledigt erklärt": models.Stationstyp.PARL_MINUS_ABLEHNUNG,
            "Verabschiedet": models.Stationstyp.PARL_MINUS_AKZEPTANZ,
        }
        
        typ = station_mapping.get(aktivitaet.get("aktivitaetsart"))
        if not typ:
            return None
            
        datum = self._parse_date(aktivitaet.get("datum"))
        return models.Station.from_dict({
            "datum": datum,
            "start_zeitpunkt": f"{datum}T00:00:00",  # Startzeitpunkt als Datum mit 00:00:00
            "gremium": aktivitaet.get("gremium", "Bundestag"),
            "dokumente": self._extract_dokumente(aktivitaet.get("drucksache")),
            "link": aktivitaet.get("fundstelle"),
            "parlament": "BT",
            "schlagworte": [],
            "stellungnahmen": [],
            "typ": typ,
            "trojaner": False,
            "betroffene_texte": []
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

    def _parse_date(self, date_str: str) -> str:
        """Konvertiert ein Datum-String in das erwartete ISO-Format"""
        if not date_str:
            return datetime.now().date().isoformat()
        try:
            return datetime.strptime(date_str, "%Y-%m-%d").date().isoformat()
        except ValueError:
            return datetime.now().date().isoformat()

