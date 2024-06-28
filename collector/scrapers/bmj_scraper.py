from scraper_interface import Scraper
from database_connector import DatabaseConnector

class BMJScraper(Scraper):

    def __init__(self, db_connector, llm_connector):
        """
        Initialize the Scraper with a database connector and an llm connector.

        Parameters:
        -----------
        db_connector : Any
            The database connector to interact with the database.
        llm_connector : Any
            The database connector to interact with the llm.
        """
        self.db_connector = db_connector
        self.llm_connector = llm_connector

    def fetch_content(self) -> str:
        return "Fetch fetch..."

    def parse_content(self):
        return

    def send_data(self, data: dict, server_url: str):
        return

