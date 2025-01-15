from openapi_client import Configuration
from pathlib import Path
import os
import logging
from collector.llm_connector import LLMConnector
from collector.scrapercache import ScraperCache
import sys

logger = logging.getLogger(__name__)

class CollectorConfiguration:
    oapiconfig : Configuration = None
    llm_connector: LLMConnector = None
    redis_host : str = None
    redis_port : int = None
    ltzfdb : str = None
    api_object_log : str = None
    scrapers_dir : Path = None
    api_key : str = None
    trojan_threshold : int = None
    cache : ScraperCache = None
    

    def __init__(self):
        global logger
        unset_keys = []
        # Database
        self.database_url = os.getenv("LTZF_DATABASE", "http://localhost:80")
        self.api_key = os.getenv("API_KEY")
        if self.api_key is None:
            unset_keys.append("API_KEY")
        
        # Caching
        self.redis_host = os.getenv("REDIS_HOST", "localhost")
        self.redis_port = int(os.getenv("REDIS_PORT", "6379"))
        self.cache = ScraperCache(self.redis_host, self.redis_port)
        
        # Scraperdir
        self.scrapers_dir = self.scrapers_dir or os.path.join(
            os.path.dirname(__file__), "scrapers"
            )
        # Thresholds and optionals
        self.trojan_threshold = int(os.getenv("TROJAN_THRESHOLD", "5"))
        self.api_obj_log = os.getenv("API_OBJ_LOG")
        
        # LLM Connector, currently only openai is supported
        self.oapiconfig = Configuration(host=self.database_url)
        if os.getenv("OPENAI_API_KEY"):
            self.llm_connector = LLMConnector.from_openai(os.getenv("OPENAI_API_KEY"))
        else:
            unset_keys.append("OPENAI_API_KEY")
        if len(unset_keys) > 0:
            logger.error(f"The following environment variables are not set: {', '.join(unset_keys)}")
            sys.exit(1)
