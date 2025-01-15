from datetime import timedelta
from openapi_client import models
from collector.convert import sanitize_for_serialization
from collector.document import Document
from pathlib import Path
from typing import Optional, Dict, Any
import logging
import redis
import sys


logger = logging.getLogger(__name__)

class ScraperCache:
    """
    Handles caching of scraped data at different levels (Gesetzesvorhaben and Dokumente).
    Provides methods to read from and write to cache, with separate directories for each level.
    """
    redis_client: Optional[redis.Redis] = None

    def __init__(self, redis_host: str, redis_port: int):
        global logger
        
        try:
            self.redis_client = redis.Redis(
                host=redis_host,
                port=redis_port,
                decode_responses=True
            )
            logger.info(f"Connected to Redis at {redis_host}:{redis_port}")
        except redis.ConnectionError as e:
            logger.error(f"Failed to connect to Redis: {e}")
            sys.exit(1)

    def store_gsvh(self, key: str, value: models.Gesetzesvorhaben):
        """Store data in either Redis or file system cache"""
        serialized = value.to_json()
        logger.debug(f"Storing gsvh {key} in redis")
        logger.debug(f"Serialized: {serialized}")
        self.redis_client.set(f"gsvh:{key}", serialized, timedelta(minutes=12))
    
    def store_dokument(self, key: str, value: Document):
        """Store data in either Redis or file system cache"""
        serialized = value.to_json()
        logger.debug(f"Storing dokument {key} in redis")
        logger.debug(f"Serialized: {serialized}")
        self.redis_client.set(f"dok:{key}", serialized)

    def get_gsvh(self, key: str) -> models.Gesetzesvorhaben:
        """Get Gesetzesvorhaben data from cache"""
        logger.debug(f"Getting gsvh {key} from cache")
        result = self.redis_client.get(f"gsvh:{key}")
        logger.debug(f"Result: {result}")
        if not result:
            return None
        return models.Gesetzesvorhaben.from_json(result)

    def get_dokument(self, key: str) -> Document:
        """Get Dokument data from cache"""
        logger.debug(f"Getting dokument {key} from cache")
        result = self.redis_client.get(f"dok:{key}")
        logger.debug(f"Result: {result}")
        if not result:
            return None
        return Document.from_json(result)

    def clear(self):
        """Clear all cache data"""
        self.redis_client.flushall()