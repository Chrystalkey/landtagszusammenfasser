from typing import List
from urllib.parse import urlsplit
from openapi_client import models

station_map = {
            "preparl-regent": "Referentenentwurf",
            "preparl-eckpup": "Eckpunktepapier",
            "parl-initiativ": "Parlamentarische Initiative",
            "parl-ausschber": "Ausschussberatung",
            "parl-vollvlsgn": "Vollversammlung",
            "parl-schlussab": "Schlussabstimmung",
            "parl-akzeptanz": "Akzeptiert",
            "parl-ablehnung": "Abgelehnt",
            "postparl-vents": "Volksentscheid",
            "postparl-gsblt": "Veröffentlicht",
            "postparl-kraft": "In Kraft"
        }

gesetzestyp_map = {
        "bgg-einspruch": "Einspruchsgesetz",
        "bgg-zustimmung": "Zustimmungsgesetz",
        "bay-parlament": "Parlamentsgesetz Bayern",
        "bay-volk": "Volksgesetzgebung Bayern",
        "sonstig": "Sonstiges"
}

doktyp_map = {
    "entwurf": "Entwurf",
    "drucksache": "Drucksache", 
    "protokoll": "Protokoll",
    "topsliste": "TOP",
    "stellungnahme": "Stellungnahme",
    "sonstig": "Sonstiges"
}

def generate_content(model: models.Gesetzesvorhaben) -> str:
    return generate_header(model) + "\n" + generate_body(model)

def generate_header(model: models.Gesetzesvorhaben) -> str:
    title = " ".join(model.titel.split())  # Normalize whitespace

    inilen = min(5, len(model.initiatoren))
    initiatoren = ", ".join(str(i) for i in sorted(model.initiatoren, key=lambda el: 0 if "(" not in el else 1)[:inilen])
    if len(model.initiatoren) > 5:
        initiatoren += ", ..."
    
    latest_station = max(model.stationen, key=lambda s: s.datum)
    last_station_type = "" + latest_station.typ # this weird construct is due to the fact that the station type is not a string but a enum class
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
            initiative =  stat.dokumente[0]
        
        if not last_drucksache or (stat.typ in ["parl-ausschber", "parl-schlussab", "postparl-gsblt"] and last_ds_date < stat.datum):
            for doc in stat.dokumente:
                if doc.typ == "drucksache":
                    last_drucksache = doc
                    last_ds_date = stat.datum
                    break

    builder = "+++\n"
    builder += f"title=\"{title}\"\n"
    builder += f"date=\"{latest_station.datum}\"\n"
    builder += "template=\"gesetzpage.html\"\n"
    builder += "[extra]\n"
    builder += f"station=\"{last_station_type}\"\n"
    builder += f"status=\"{status}\"\n"
    builder += f"date=\"{latest_station.datum}\"\n"
    builder += f"initiator=\"{initiatoren}\"\n"
    builder += f"gesetzestyp=\"{gesetzestyp_map.get("" + model.typ, "" + model.typ)}\"\n"
    if last_drucksache:
        builder += f"drucksache_link=\"{last_drucksache.link}\"\n"
    if initiative:
        builder += f"entwurf_link=\"{initiative.link}\"\n"
    builder += f"authoren=\"{last_drucksache.autoren}\"\n"
    builder += f"texte=\"{latest_station.betroffene_texte}\"\n"
    builder += "+++\n"
    return builder

def generate_body(model: models.Gesetzesvorhaben) -> str:
    global doktyp_map
    title = model.titel
    last_station = max(model.stationen, key=lambda s: s.datum)
    last_station_type = ""+last_station.typ

    last_drucksache = None
    last_ds_date = None
    initiative = None
    for stat in model.stationen:
        if not initiative and stat.typ == "parl-initiativ":
            initiative =  stat.dokumente[0]
        
        if not last_drucksache or (stat.typ in ["parl-ausschber", "parl-schlussab", "postparl-gsblt"] and last_ds_date < stat.datum):
            for doc in stat.dokumente:
                if doc.typ == "drucksache":
                    last_drucksache = doc
                    last_ds_date = stat.datum
                    break
    last_stype_readable = station_map.get(last_station_type, last_station_type)

    builder = f"# {title}\n\n"
    
    builder += "## Kurzzusammenfassung\n"
    zusammenfassung = "Keine Zusammenfassung verfügbar."
    if last_drucksache.zusammenfassung:
        zusammenfassung = last_drucksache.zusammenfassung
    builder += f"{zusammenfassung}\n\n"
    
    builder += "## Beratungsverlauf\n"

    sorted_stations = sorted(model.stationen, key=lambda x: (x.datum, 0 if x.typ.startswith("prep") else 1 if x.typ.startswith("parl") else 2))
    for s in sorted_stations:
        if s.typ == "parl-ausschber":
            builder += "### " + station_map.get(s.typ) + f" im {s.gremium}" + "\n"
        else:
            builder += "### " + station_map.get(s.typ, "Unbekannte Station") + "\n"
        builder += f"Datum: {s.datum}\n\n"
        if len(s.dokumente) > 0: 
            builder += "#### Dokumente\n"
        for dok in s.dokumente:
            builder += f"- [{dok.titel if dok.titel != "None" else "" } ({doktyp_map.get(dok.typ, "Sonstiges")})]({dok.link})\n"
    
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
tables=[{name="Initiative", stations=["parl-initiativ"]},{name="Ausschussberatung", stations=["parl-ausschber", "parl-vollvlsg"]}]\n"""
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
tables=[{name="Abgelehnt", stations=["parl-abgelehnt"]},{name="Angenommen", stations=["parl-angenommen"]},{name="Veröffentlicht", stations=["postparl-vents", "postparl-gsblt", "postparl-kraft"]}]\n"""
    builder += f"laws={gesetze}\n"
    builder += "+++\n"
    return builder
