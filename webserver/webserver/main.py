"""
Web server that periodically:
1. Fetches latest data from the database via OAPI collector
2. Updates data in the webserver directory
3. Runs zola build
4. Copies generated files to webserver directory
5. Restarts the webserver
"""

from pathlib import Path
import logging
import os
import subprocess
import threading
import time
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from typing import List, Optional

from openapi_client import ApiClient, Configuration, DefaultApi
from openapi_client import models

logger = logging.getLogger(__name__)

class ContentGenerator:
    def __init__(self):
        self.base_dir = Path("zolasite/content/gesetze")
        
    def generate_md(self, gsvh: models.Gesetzesvorhaben, timestamp: str, last_state: str) -> str:
        """Generate markdown content for a legislative proposal."""
        title = " ".join(gsvh.titel.split())  # Normalize whitespace
        
        # Map certain parliamentary states to ausschber
        if last_state in {"parl-vollvlsgn", "parl-schlussab", "parl-ggentwurf"}:
            display_state = "parl-ausschber"
        else:
            display_state = last_state
            
        initiatoren = ", ".join(str(i) for i in gsvh.initiatoren)
        
        return f"""+++
title="{title}"
date="{timestamp}"
[extra]
state="{display_state}"
extra_img="/icons/90daytimeout.png"
scenario_img="/images/Szenario10.png"
initiator="{initiatoren}"
+++
"""

    def generate_content(self, response: Optional[List[models.Response]]) -> None:
        """Generate markdown files from legislative proposals."""
        if not response:
            logger.warning("No items to generate")
            return

        for gsvh in response.payload:
            # Find latest station
            latest_station = max(gsvh.stationen, key=lambda s: s.zeitpunkt)
            station_type = latest_station.typ

            # Determine output path based on station type
            if station_type.startswith("preparl"):
                subdir = "vorbereitung"
            elif station_type.startswith("parl"):
                subdir = "parlament"
            elif station_type.startswith("postparl"):
                subdir = "postparlament"
            else:
                logger.warning(f"Unknown station type: {station_type}")
                continue

            path = self.base_dir / subdir / f"{gsvh.api_id}.md"
            content = self.generate_md(gsvh, latest_station.zeitpunkt, station_type)
            
            logger.info(f"Writing to {path}")
            path.write_text(content, encoding="utf-8")


class WebServer:
    def __init__(self, port: int):
        self.directory = Path(__file__).parent.parent
        self.content_generator = ContentGenerator()
        self.port = port

    def update_data(self) -> None:
        """Fetch latest data from API and generate content."""
        logging.basicConfig(level=logging.DEBUG)
        last_update = "2024-01-01"  # TODO: Make this dynamic

        config = Configuration(
            host=f"http://{os.environ['LTZFDB_HOST']}:{int(os.environ['LTZFDB_PORT'])}"
        )

        try:
            with ApiClient(config) as api_client:
                api = DefaultApi(api_client)
                response = api.api_v1_gesetzesvorhaben_get(
                    updated_since=last_update,
                    limit=100,
                    parlament=models.Parlament.BY
                )
                self.content_generator.generate_content(response)
        except Exception as e:
            logger.error(f"Failed to fetch data: {e}")

    def build_site(self) -> None:
        """Build the static site using zola."""
        os.chdir(self.directory)
        subprocess.run(["zola", "build", "-o", "../www", "--force"], cwd="zolasite")
        os.chdir(self.directory / "www")

    def run(self) -> None:
        """Run the web server with periodic updates."""
        self.update_data()
        self.build_site()

        server = ThreadingHTTPServer(("", self.port), SimpleHTTPRequestHandler)
        server_thread = threading.Thread(target=server.serve_forever)
        server_thread.start()

        try:
            while True:
                time.sleep(100)
                self.update_data()
                
                logger.info("Restarting server...")
                server.shutdown()
                server.server_close()
                server_thread.join()

                self.build_site()

                server = ThreadingHTTPServer(("", self.port), SimpleHTTPRequestHandler)
                server_thread = threading.Thread(target=server.serve_forever)
                server_thread.start()
                
        except KeyboardInterrupt:
            logger.info("Server stopped by user")
            server.shutdown()
            server.server_close()
            server_thread.join()


if __name__ == "__main__":
    port = int(80 if os.environ["PORT"] is None else os.environ["PORT"])
    server = WebServer(port)
    server.run()
