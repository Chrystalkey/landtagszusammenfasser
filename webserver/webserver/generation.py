from openapi_client import models

station_map = {
            "preparl-regent": "Referentenentwurf",
            "preparl-eckpup": "Eckpunktepapier",
            "parl-initiativ": "parlamentarische Initiative",
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
        "volksgg": "Volksgesetzgebung",
        "landgg": "Landesgesetz",
        "sonstig": "Sonstiges"
}

def generate_content(model: models.Gesetzesvorhaben) -> str:
    return generate_header(model) + "\n" + generate_body(model)

def generate_header(model: models.Gesetzesvorhaben) -> str:
    title = " ".join(model.titel.split())  # Normalize whitespace
    initiatoren = ", ".join(str(i) for i in model.initiatoren)
    
    latest_station = max(model.stationen, key=lambda s: s.zeitpunkt)
    last_station_type = ""+latest_station.typ # this weird construct is due to the fact that the station type is not a string but a enum class
    status = "Unbekannt"
    if last_station_type.startswith("preparl"):
        status = "In Vorbereitung"
    elif last_station_type.startswith("parl") and last_station_type not in ["parl-akzeptanz", "parl-ablehnung"]:
        status = "In Beratung"
    elif last_station_type.startswith("postparl") or last_station_type in ["parl-akzeptanz", "parl-ablehnung"]:
        status = "In Nachbereitung"

    builder = "+++\n"
    builder += f"title=\"{title}\"\n"
    builder += "template=\"gesetzpage.html\"\n"
    builder += "[extra]\n"
    builder += f"station=\"{last_station_type}\"\n"
    builder += f"status=\"{status}\"\n"
    builder += f"date=\"{latest_station.zeitpunkt}\"\n"
    builder += f"initiator=\"{initiatoren}\"\n"
    builder += f"gesetzestyp=\"{gesetzestyp_map.get("" + model.typ, "" + model.typ)}\"\n"
    for doc in latest_station.dokumente:
        if doc.typ == "drucksache":
            builder += f"drucksache_link=\"{doc.url}\"\n"
        elif doc.typ == "entwurf":
            builder += f"entwurf_link=\"{doc.url}\"\n"
    builder += "+++\n"
    return builder

def generate_body(model: models.Gesetzesvorhaben) -> str:
    title = " ".join(model.titel.split())  # Normalize whitespace
    last_station = max(model.stationen, key=lambda s: s.zeitpunkt)
    last_station_type = ""+last_station.typ
    last_stype_readable = station_map.get(last_station_type, last_station_type)

    builder = f"# {title}\n\n"
    
    builder += "## Kurzzusammenfassung\n"
    zusammenfassung = "Keine Zusammenfassung verfügbar."
    for doc in last_station.dokumente:
        if doc.zusammenfassung:
            zusammenfassung = doc.zusammenfassung
            break
    builder += f"{zusammenfassung}\n\n"
    
    builder += "## Beratungsverlauf\n"
    builder += "### Initiative\n"
    # ... rest of your beratungsverlauf content ...
    
    builder += "\n## Weiterführende Links\n"
    for doc in last_station.dokumente:
        builder += f"- [{doc.titel}]({doc.url})\n"
    
    if model.links:
        for link in model.links:
            builder += f"- [Link]({link})\n"
    
    return builder

