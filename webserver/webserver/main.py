# every x minutes:
# - get the latest data from the database with the oapi collector
# - update the date in the webserver directory
# - run zola build
# - copy the generated files to the webserver directory
# - restart the webserver

from openapi_client import Configuration
from openapi_client import models
import openapi_client
import signal
import logging
import os
import subprocess
import threading
import time
from http.server import SimpleHTTPRequestHandler
from socketserver import TCPServer
import shutil

# Configuration
logger = logging.getLogger(__name__)
web_directory = "www"
port = 8081


def update_main():
    global logger
    logging.basicConfig(level=logging.DEBUG)
    last_update = "2024-01-01"

    oapiconfig = Configuration(host=f"http://{os.environ['LTZFDB_HOST']}:{int(os.environ['LTZFDB_PORT'])}")

    with openapi_client.ApiClient(oapiconfig) as api_client:
        api_instance = openapi_client.DefaultApi(api_client)
        try:
            response = api_instance.api_v1_gesetzesvorhaben_get(
                updated_since=last_update, limit=100, parlament=models.Parlament.BY
            )
        except openapi_client.ApiException as e:
            logger.error(
                f"Exception when calling DefaultApi->api_v1_gesetzesvorhaben_get: {e}"
            )
    generate_content(response)


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
            path = f"zolasite/content/gesetze/postparlament/{gsvh.api_id}.md"
        else:
            logger.warning(f"Unknown type {gsvh.typ}")
            continue
        station_type = ""+latest_station.typ
        text = generate_md(gsvh, latest_station.zeitpunkt, station_type)
        logger.info(f"Writing to {path}")
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)


def generate_md(gsvh: models.Gesetzesvorhaben, timestamp: str, last_state: str) -> str:
    text = "+++\n"
    title: str = gsvh.titel
    title = (
        title.replace("\n", " ").replace("\r\n", " ").replace("\r", " ").strip()
    )
    text += f'title="{title}"\n'
    text += f'date="{timestamp}"\n'
    text += "[extra]\n"
    if last_state in ["parl-vollvlsgn", "parl-schlussab", "parl-ggentwurf"]:
        text += f'state="parl-ausschber"\n'
    else:
        text += f'state="{str(last_state)}"\n'
    text += 'extra_img="/icons/90daytimeout.png"\n'
    text += 'scenario_img="/images/Szenario10.png"\n'
    initiatoren = ""
    for i in gsvh.initiatoren:
        initiatoren += str(i) + ", "
    initiatoren = initiatoren[:-2]
    text += f'initiator="{initiatoren}"\n'
    text += "+++\n\n"

    return text


import os
import subprocess
import time
from http.server import SimpleHTTPRequestHandler
from http.server import ThreadingHTTPServer
from threading import Thread

DIRECTORY = os.path.dirname(os.path.realpath(__file__))


# Function that runs a subprocess
def run_subprocess():
    global DIRECTORY
    print(f"Running subprocess in {DIRECTORY}")
    # Replace with your actual command
    os.chdir(DIRECTORY)
    subprocess.run(["zola", "build", "-o", "../www", "--force"], cwd="zolasite")
    os.chdir(os.path.join(DIRECTORY, "www"))


def run_server():
    # Initialize with subprocess
    update_main()  # Call the update function
    run_subprocess()

    # Set up the server
    server_address = ("", int(os.environ["PORT"]))
    httpd = ThreadingHTTPServer(server_address, SimpleHTTPRequestHandler)

    # Start the server in a new thread
    server_thread = Thread(target=httpd.serve_forever)
    server_thread.start()

    # Periodically call update_main and restart server every 10 seconds
    try:
        while True:
            time.sleep(100)
            update_main()
            print("Stopping server for restart...")
            httpd.server_close()  # Stop the server
            httpd.shutdown()

            # Wait for the server thread to stop
            server_thread.join()

            # Run the subprocess again (step 1)
            run_subprocess()

            # Restart the server
            print("Restarting server...")
            httpd = ThreadingHTTPServer(server_address, SimpleHTTPRequestHandler)
            server_thread = Thread(target=httpd.serve_forever)
            server_thread.start()
    except KeyboardInterrupt:
        print("Server stopped by user.")
        httpd.server_close()
        httpd.shutdown()
        server_thread.join()


if __name__ == "__main__":
    run_server()
