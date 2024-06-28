from database_connector import DatabaseConnector
from llm_connector import LLMConnector
from scraper_interface import Scraper
import os
import importlib.util

scrapers_dir = "./scrapers"

def main():
    print ("Starting collector manager.")
    db_connector = DatabaseConnector()
    llm_connector = LLMConnector()

    # Load all the scrapers from the scrapers dir
    scrapers = load_scrapers(db_connector, llm_connector)

    for scraper in scrapers:
        print(f"Running scraper: {scraper.__class__.__name__}")
        try:
            # Actually run the scraper instance
            print(scraper.fetch_content())

        except Exception as e:
            print(f"Error running scraper {scraper.__class__.__name__}: {e}")

def load_scrapers(db_connector, llm_collector):
    scrapers = []
    for filename in os.listdir(scrapers_dir):
        if filename.endswith("_scraper.py"):
            module_name = filename[:-3]
            module_path = os.path.join(scrapers_dir, filename)
            spec = importlib.util.spec_from_file_location(module_name, module_path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            for attr in dir(module):
                cls = getattr(module, attr)
                if isinstance(cls, type) and issubclass(cls, Scraper) and cls is not Scraper:
                    scrapers.append(cls(db_connector, llm_collector))
    return scrapers

if __name__ == "__main__":
    main()
