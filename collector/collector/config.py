from openapi_client import Configuration
from pathlib import Path
import os
import logging

logger = logging.getLogger(__name__)

class CollectorConfiguration:
    oapiconfig : Configuration = None
    redis_host : str = None
    redis_port : int = None
    ltzfdb : str = None
    api_object_log: str = None
    scrapers_dir :Path = None

    def __init__(self):
        global logger
        self.ltzfdb_host = os.environ.get("LTZF_DATABASE", "http://localhost:80")
        self.redis_host = os.environ.get("REDIS_HOST", "localhost")
        self.redis_port = int(os.environ.get("REDIS_PORT", 6379))
        strpath = os.environ.get("API_OBJ_LOG", None)
        if strpath:
            self.api_object_log = Path(strpath)
            self.api_object_log.mkdir(parents=True, exist_ok=True)
        
        self.oapiconfig = Configuration(host=self.ltzfdb_host)
        self.scrapers_dir = Path("./collector/scrapers")
        logger.debug(f"Finished Configuration with values {str(self)}")