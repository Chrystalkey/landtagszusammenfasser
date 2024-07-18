from scraper_interface import Scraper
from database_connector import DatabaseConnector
from bs4 import BeautifulSoup
import requests

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
        self.url = "https://www.justiz.bayern.de/ministerium/gesetzgebung/"

    def fetch_content(self) -> str:
        return "Fetch fetch..."

    def parse_content(self):
        content = requests.get(self.url)
        soup = BeautifulSoup(content.text, 'html.parser')
        diskussionsentwurfe_h2 = soup.find('h2', id='jump_0_13')
        diskussionsentwuerfe = []
        next_element = diskussionsentwurfe_h2.find_next_sibling()
        while next_element and "info-box" in next_element.get("class", []) and next_element.name == "div":
            diskussionsentwuerfe.append(next_element)
            next_element = next_element.find_next_sibling()
        

    def send_data(self, data: dict, server_url: str):
        return

