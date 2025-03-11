from typing import List
from urllib.parse import urlsplit
from datetime import datetime
from openapi_client import models

station_map = {
    "preparl-regent": "Referentenentwurf",
    "preparl-eckpup": "Eckpunktepapier",
    "preparl-regbsl": "Kabinettsbeschluss",
    "preparl-vbegde": "Volksbegehren/Diskussionsentwurf",
    "parl-initiativ": "Parlamentarische Initiative",
    "parl-ausschber": "Ausschussberatung",
    "parl-vollvlsgn": "Vollversammlung",
    "parl-akzeptanz": "Akzeptiert",
    "parl-ablehnung": "Abgelehnt",
    "parl-ggentwurf": "Gegenentwurf des Parlaments",
    "postparl-vesja": "Volksentscheid (Ja)",
    "postparl-vesne": "Volksentscheid (Nein)",
    "postparl-gsblt": "Veröffentlicht",
    "postparl-kraft": "In Kraft",
    "sonstig": "Sonstiges"
}

gesetzestyp_map = {
    "gg-einspruch": "Einspruchsgesetz",
    "gg-zustimmung": "Zustimmungsgesetz",
    "gg-land-parl": "Parlamentsgesetz",
    "gg-land-volk": "Volksgesetzgebung",
    "sonstig": "Sonstiges"
}

doktyp_map = {
    "preparl-entwurf": "Vorparlamentarischer Entwurf", 
    "entwurf": "Gesetzesentwurf",
    "beschlussempf": "Beschlussempfehlung",
    "stellungnahme": "Stellungnahme",
    "plenar-protokoll": "Plenarprotokoll",
    "plenar-tops": "Plenarsitzung TOPs",
    "as-tops": "Ausschusssitzung TOPs",
    "as-tops-aend": "Ausschusssitzung TOPs (Änderung)",
    "as-tops-ergz": "Ausschusssitzung TOPs (Ergänzung)",
    "sonstig": "Sonstiges"
}

def format_datetime(dt):
    """Format a datetime object for display"""
    if isinstance(dt, datetime):
        return dt.strftime("%Y-%m-%d")
    return str(dt)

def generate_content(model: models.Vorgang) -> str:
    return generate_header(model) + "\n" + generate_body(model)

def generate_header(model: models.Vorgang) -> str:
    title = " ".join(model.titel.split())  # Normalize whitespace

    inilen = min(5, len(model.initiatoren))
    initiatoren = ", ".join(str(i) for i in sorted(model.initiatoren, key=lambda el: 0 if "(" not in el else 1)[:inilen])
    if len(model.initiatoren) > 5:
        initiatoren += ", ..."
    
    latest_station = max(model.stationen, key=lambda s: s.start_zeitpunkt)
    last_station_type = latest_station.typ
    status = "Unbekannt"
    if last_station_type.startswith("preparl"):
        status = "In Vorbereitung"
    elif last_station_type.startswith("parl") and last_station_type not in ["parl-akzeptanz", "parl-ablehnung"]:
        status = "In Beratung"
    elif last_station_type.startswith("postparl") or last_station_type in ["parl-akzeptanz", "parl-ablehnung"]:
        status = "In Nachbereitung"
    
    initiative = None
    last_drucksache = None
    last_ds_date = None
    for stat in model.stationen:
        if not initiative and stat.typ == "parl-initiativ":
            if stat.dokumente:
                initiative = stat.dokumente[0].actual_instance
        
        if not last_drucksache or (stat.typ in ["parl-ausschber", "parl-akzeptanz", "postparl-gsblt"] and (not last_ds_date or last_ds_date < stat.start_zeitpunkt)):
            for doc in stat.dokumente:
                if doc.actual_instance.typ == "entwurf":
                    last_drucksache = doc.actual_instance
                    last_ds_date = stat.start_zeitpunkt
                    break

    builder = "+++\n"
    builder += f"title=\"{title}\"\n"
    builder += f"date=\"{format_datetime(latest_station.start_zeitpunkt)}\"\n"
    builder += "template=\"gesetzpage.html\"\n"
    builder += "[extra]\n"
    builder += f"station=\"" + last_station_type + "\"\n"
    builder += f"status=\"{status}\"\n"
    builder += f"date=\"{format_datetime(latest_station.start_zeitpunkt)}\"\n"
    builder += f"initiator=\"{initiatoren}\"\n"
    builder += f"gesetzestyp=\"{gesetzestyp_map.get(model.typ, model.typ)}\"\n"
    
    if last_drucksache:
        builder += f"drucksache_link=\"{last_drucksache.link}\"\n"
        if hasattr(last_drucksache, 'drucksnr') and last_drucksache.drucksnr:
            builder += f"drucksnr=\"{last_drucksache.drucksnr}\"\n"
        if hasattr(last_drucksache, 'autoren') and last_drucksache.autoren:
            builder += f"authoren=\"{', '.join(last_drucksache.autoren)}\"\n"
    
    if initiative:
        builder += f"entwurf_link=\"{initiative.link}\"\n"
    
    if hasattr(latest_station, 'betroffene_texte') and latest_station.betroffene_texte:
        builder += f"texte=\"{', '.join(latest_station.betroffene_texte)}\"\n"
    
    builder += "+++\n"
    return builder

def generate_body(model: models.Vorgang) -> str:
    global doktyp_map
    title = model.titel
    last_station = max(model.stationen, key=lambda s: s.start_zeitpunkt)
    last_station_type = last_station.typ

    last_drucksache = None
    last_ds_date = None
    initiative = None
    for stat in model.stationen:
        if not initiative and stat.typ == "parl-initiativ":
            if stat.dokumente:
                initiative = stat.dokumente[0].actual_instance
        
        if not last_drucksache or (stat.typ in ["parl-ausschber", "parl-akzeptanz", "postparl-gsblt"] and (not last_ds_date or last_ds_date < stat.start_zeitpunkt)):
            for doc in stat.dokumente:
                if doc.actual_instance.typ == "entwurf":
                    last_drucksache = doc.actual_instance
                    last_ds_date = stat.start_zeitpunkt
                    break
    
    last_stype_readable = station_map.get(last_station_type, last_station_type)

    builder = f"# {title}\n\n"
    
    builder += "## Kurzzusammenfassung\n"
    zusammenfassung = "Keine Zusammenfassung verfügbar."
    if last_drucksache and hasattr(last_drucksache, 'zusammenfassung') and last_drucksache.zusammenfassung:
        zusammenfassung = last_drucksache.zusammenfassung
    builder += f"{zusammenfassung}\n\n"
    
    builder += "## Beratungsverlauf\n"

    # Sort by start_zeitpunkt as datetime objects
    sorted_stations = sorted(model.stationen, key=lambda x: (x.start_zeitpunkt, 0 if x.typ.startswith("prep") else 1 if x.typ.startswith("parl") else 2))
    for s in sorted_stations:
        if s.typ == "parl-ausschber" and hasattr(s, 'gremium') and s.gremium:
            builder += "### " + station_map.get(s.typ) + f" im {s.gremium.name}" + "\n"
        else:
            builder += "### " + station_map.get(s.typ, "Unbekannte Station") + "\n"
        # Format the datetime for display
        builder += f"Datum: {format_datetime(s.start_zeitpunkt)}\n\n"
        if len(s.dokumente) > 0: 
            builder += "#### Dokumente\n"
        for dok in s.dokumente:
            d = dok.actual_instance
            title_text = d.titel if d.titel != "None" else ""
            dok_type = doktyp_map.get(d.typ, "Sonstiges")
            builder += f"- [{title_text} ({dok_type})]({d.link})\n"
    
    builder += "\n## Weiterführende Links\n"
    if model.links:
        for link in model.links:
            urls = urlsplit(link)
            builder += f"- [{urls.netloc}]({link})"
            if link.endswith(".pdf"):
                builder += " (PDF)\n"
            else:
                builder += " (Website)\n"
    
    return builder

def generate_beratung(gesetze: List[str]) -> str:
    builder = """+++
title="In Beratung"
template="categorypage.html"
[extra]
tables=[{name="Initiative", stations=["parl-initiativ"]},{name="Ausschussberatung", stations=["parl-ausschber", "parl-vollvlsgn"]}]\n"""
    builder += f"laws={gesetze}\n"
    builder += "+++\n"
    return builder


def generate_vorbereitung(gesetze: List[str]) -> str:
    builder = """+++
title="In Vorbereitung"
template="categorypage.html"
[extra]
tables=[{name="Diskussionsentwurf", stations=["preparl-regent"]}, {name="Eckpunktepapier", stations=["preparl-eckpup"]}]\n"""
    builder += f"laws={gesetze}\n"
    builder += "+++\n"
    return builder

def generate_nachbereitung(gesetze: List[str]) -> str:
    builder = """+++
title="In Nachbereitung"
template="categorypage.html"
[extra]
tables=[{name="Abgelehnt", stations=["parl-ablehnung"]},{name="Angenommen", stations=["parl-akzeptanz"]},{name="Veröffentlicht", stations=["postparl-vesja", "postparl-vesne", "postparl-gsblt", "postparl-kraft"]}]\n"""
    builder += f"laws={gesetze}\n"
    builder += "+++\n"
    return builder
