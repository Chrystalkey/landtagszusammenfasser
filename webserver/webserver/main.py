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
import toml
import webserver.generation as generation
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from typing import List, Optional

from openapi_client import ApiClient, Configuration, DefaultApi
from openapi_client import models

logging.basicConfig(
    level=logging.INFO, format="%(asctime)s|%(levelname)s: %(filename)s: %(message)s"
)

logger = logging.getLogger(__name__)


class ContentGenerator:
    def __init__(self):
        self.base_dir = Path().cwd() / Path("./zolasite/content")

    def generate_content(self, response: Optional[List[models.Vorgang]]) -> None:
        """Generate markdown files from legislative proposals."""

        pagestatepath = self.base_dir / "pagestate.toml"
        pagestate = {}
        if pagestatepath.exists():
            pagestate = toml.loads(pagestatepath.read_text(encoding="utf-8"))
        else:
            pagestate = {"beratung": [], "nachbereitung": [], "vorbereitung": []}
        if not response:
            logger.warning("No items to generate")

            try:
                ber_path = Path(self.base_dir / "beratung.md")
                ber_path.write_text(
                    generation.generate_beratung(pagestate["beratung"]),
                    encoding="utf-8",
                )
                vorb_path = Path(self.base_dir / "vorbereitung.md")
                vorb_path.write_text(
                    generation.generate_vorbereitung(pagestate["vorbereitung"]),
                    encoding="utf-8",
                )
                nach_path = Path(self.base_dir / "nachbereitung.md")
                nach_path.write_text(
                    generation.generate_nachbereitung(pagestate["nachbereitung"]),
                    encoding="utf-8",
                )

                pagestatepath.write_text(toml.dumps(pagestate))
            except Exception as e:
                logger.error(f"Failed to write to pagestate or the section sites: {e}")
            return

        # Handle the updated response structure
        vorgaenge = (
            response if isinstance(response, list) else getattr(response, "payload", [])
        )

        for vg in vorgaenge:
            # Find latest station using zp_start instead of datum
            latest_station = max(vg.stationen, key=lambda s: s.zp_start)
            station_type = latest_station.typ

            # Determine output path based on station type
            if station_type.startswith("preparl"):
                if vg.api_id not in pagestate["vorbereitung"]:
                    pagestate["vorbereitung"].append(vg.api_id)
            elif station_type.startswith("parl") and station_type not in [
                "parl-ablehnung",
                "parl-akzeptanz",
            ]:
                if vg.api_id not in pagestate["beratung"]:
                    pagestate["beratung"].append(vg.api_id)
            elif station_type.startswith("postparl") or station_type in [
                "parl-ablehnung",
                "parl-akzeptanz",
            ]:
                if vg.api_id not in pagestate["nachbereitung"]:
                    pagestate["nachbereitung"].append(vg.api_id)
            else:
                logger.warning(f"Unknown station type: {station_type}")
                continue

            path = self.base_dir / "gesetze" / f"{vg.api_id}.md"
            content = generation.generate_content(vg)
            try:
                logger.info(f"Updating `{path}`")
                if path.exists():
                    existing_content = path.read_text(encoding="utf-8")
                    if existing_content == content:
                        logger.info(
                            f"Content for {vg.api_id} already exists and is unchanged"
                        )
                        continue
                    else:
                        logger.info(
                            f"Content for {vg.api_id} already exists but is different"
                        )
                        os.remove(str(path))
                os.makedirs(path.parent, exist_ok=True)  # Ensure directory exists
                with path.open("w", encoding="utf-8") as file:
                    file.write(content)
            except Exception as e:
                logger.error(f"Failed to write to {path}: {e}")

        try:
            ber_path = Path(self.base_dir / "beratung.md")
            ber_path.write_text(
                generation.generate_beratung(pagestate["beratung"]), encoding="utf-8"
            )
            vorb_path = Path(self.base_dir / "vorbereitung.md")
            vorb_path.write_text(
                generation.generate_vorbereitung(pagestate["vorbereitung"]),
                encoding="utf-8",
            )
            nach_path = Path(self.base_dir / "nachbereitung.md")
            nach_path.write_text(
                generation.generate_nachbereitung(pagestate["nachbereitung"]),
                encoding="utf-8",
            )

            pagestatepath.write_text(toml.dumps(pagestate))
        except Exception as e:
            logger.error(f"Failed to write to pagestate or the section sites: {e}")


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
            host=f"http://{os.environ.get('LTZF_API_HOST')}:{os.environ.get('LTZF_API_PORT', 80)}"
        )

        try:
            with ApiClient(config) as api_client:
                api = DefaultApi(api_client)
                # Update the API call from gsvh_get to vorgang_get
                response = api.vorgang_get()
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
                time.sleep(30)
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
