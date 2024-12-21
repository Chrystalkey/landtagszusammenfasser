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
import webserver.generation as generation
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from typing import List, Optional

from openapi_client import ApiClient, Configuration, DefaultApi
from openapi_client import models

logger = logging.getLogger(__name__)

class ContentGenerator:
    def __init__(self):
        self.base_dir = Path("zolasite/content")

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
                subdir = "in-vorbereitung"
            elif station_type.startswith("parl") and station_type not in ["parl-abgelehnt", "parl-akzeptiert"]:
                subdir = "in-beratung"
            elif station_type.startswith("postparl") or station_type in ["parl-abgelehnt", "parl-akzeptiert"]:
                subdir = "in-nachbereitung"
            else:
                logger.warning(f"Unknown station type: {station_type}")
                continue

            path = self.base_dir / subdir / f"{gsvh.api_id}.md"
            content = generation.generate_content(gsvh)
            
            logger.info(f"Writing to {path}")
            if path.exists():
                with open(path, "r", encoding="utf-8") as file:
                    existing_content = file.read()
                if existing_content == content:
                    logger.info(f"Content for {gsvh.api_id} already exists and is unchanged")
                    continue
                else:
                    logger.info(f"Content for {gsvh.api_id} already exists but is different")
                    os.remove(path)
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
            host=f"http://{os.environ.get('LTZFDB_HOST')}:{os.environ.get('LTZFDB_PORT', 80)}"
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
    port = int(os.environ.get("PORT", 80))
    server = WebServer(port)
    server.run()
