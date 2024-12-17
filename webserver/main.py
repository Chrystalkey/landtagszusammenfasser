# every x minutes: 
# - get the latest data from the database with the oapi collector
# - update the date in the webserver directory
# - run zola build
# - copy the generated files to the webserver directory
# - restart the webserver

from openapi_client import Configuration
from openapi_client import models
import openapi_client
import logging

logger = logging.getLogger(__name__)

def thread_main():
    global logger
    logging.basicConfig(level=logging.DEBUG)
    last_update = "2024-01-01"
    gsvhs: [models.Gesetzesvorhaben] = []
    
    oapiconfig = Configuration(host="http://localhost:8080")
    
    with openapi_client.ApiClient(oapiconfig) as api_client:
        api_instance = openapi_client.DefaultApi(api_client)
        try:
            response = api_instance.api_v1_gesetzesvorhaben_get(
                updated_since=last_update,
                limit=100,
                parlament=models.Parlament.BY
            )
        except openapi_client.ApiException as e:
            logger.error(f"Exception when calling DefaultApi->api_v1_gesetzesvorhaben_get: {e}")
    generate_content(response)
    build_website()
    run_server()
    
def run_server():
    import subprocess
    subprocess.run(["python", "-m", "http.server", "8081"], cwd="www")

def build_website():
    # run zola build
    import subprocess
    import os
    import shutil
    # build website
    subprocess.run(["zola", "build"], cwd="zolasite")
    # copy generated files to webserver directory
    if os.path.exists("www"):
        shutil.rmtree("www")
    shutil.copytree("zolasite/public", "www")
    

# take the vec of gsvh and convert them to neat little .md files in the webserver directory
def generate_content(response: [models.Response]):
    global logger
    if response is None:
        logger.warning("No items to generate")
    
    for gsvh in response.payload:
        path = ""
        latest_station_type = None
        latest_station = None
        for station in gsvh.stationen:
            if latest_station is None or station.zeitpunkt > latest_station.zeitpunkt:
                latest_station = station
                latest_station_type = station.typ
        if latest_station_type.startswith("preparl"):
            path = f"zolasite/content/gesetze/vorbereitung/{gsvh.api_id}.md"
        elif latest_station_type.startswith("parl"):
            path = f"zolasite/content/gesetze/parlament/{gsvh.api_id}.md"
        elif latest_station_type.startswith("postparl"):
            path = f"zolasite/content/gesetze/abgeschlossen/{gsvh.api_id}.md"
        else:
            logger.warning(f"Unknown type {gsvh.typ}")
            continue
        text = generate_md(gsvh, latest_station.zeitpunkt)
        logger.info(f"Writing to {path}")
        logger.debug(text)
        logger.debug(text[540:])
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)

def generate_md(gsvh: models.Gesetzesvorhaben, timestamp: str)->str:
    text = "+++\n"
    text += f"title=\"{gsvh.titel}\"\n"
    text += f"date=\"{timestamp}\"\n"
    text += "[extra]\n"
    text += f"state=\"eckpunkt\"\n" # todo: change when the template is ready
    text += "extra_img=\"/icons/90daytimeout.png\"\n"
    text += "scenario_img=\"/images/Szenario10.png\"\n"
    initiatoren = ""
    for i in gsvh.initiatoren:
        initiatoren += str(i) + ", "
    initiatoren = initiatoren[:-2]
    text += f"initiator=\"{initiatoren}\"\n"
    text += "+++\n\n"

    return text


if __name__ == "__main__":
    thread_main()