from collector.interface import Scraper
import os
import importlib.util
from openapi_client import Configuration

scrapers_dir = "./scrapers"

def main():
    print ("Starting collector manager.")

    # Load all the scrapers from the scrapers dir
    oapiconfig = Configuration(host="http://localhost:8080")

    scrapers: list[Scraper] = load_scrapers(oapiconfig)

    for scraper in scrapers:
        print(f"Running scraper: {scraper.__class__.__name__}")
        try:
            # Actually run the scraper instance
            scraper.extract()
            scraper.send()

        except Exception as e:
            print(f"Error running scraper {scraper.__class__.__name__}: {e}")

def load_scrapers(config):
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
                    scrapers.append(cls(config))
    return scrapers

if __name__ == "__main__":
    main()
