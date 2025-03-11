from datetime import timedelta
import json
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
    Handles caching of scraped data at different levels (Vorgang and Dokumente).
    Provides methods to read from and write to cache using Redis.
    """
    redis_client: Optional[redis.Redis] = None
    cache_expiry_minutes: int = 60 * 24  # Default 24 hours for document cache
    disabled: bool = False

    def __init__(self, redis_host: str, redis_port: int, doc_cache_expiry_minutes: int = None, disabled: bool = False):
        global logger
        self.disabled = disabled
        if disabled or redis_host is None or redis_port is None:
            self.disabled = True
            logger.warning("Cacheing disabled")
            return
        
        if doc_cache_expiry_minutes:
            self.cache_expiry_minutes = doc_cache_expiry_minutes
        
        try:
            self.redis_client = redis.Redis(
                host=redis_host,
                port=redis_port,
                decode_responses=True
            )
            # Test connection
            self.redis_client.ping()
            logger.info(f"Connected to Redis at {redis_host}:{redis_port}")
        except redis.ConnectionError as e:
            logger.error(f"Failed to connect to Redis: {e}")
            sys.exit(1)
        except Exception as e:
            logger.error(f"Unexpected error connecting to Redis: {e}")
            sys.exit(1)

    def store_gsvh(self, key: str, value: models.Vorgang):
        """Store Vorgang data in Redis cache"""
        if self.disabled:
            return True
        try:
            serialized = json.dumps(sanitize_for_serialization(value))
            logger.debug(f"Storing vorgang {key} in redis")
            self.redis_client.set(f"vg:{key}", serialized, timedelta(minutes=12))
            return True
        except Exception as e:
            logger.error(f"Error storing vorgang {key} in cache: {e}")
            return False
    
    def store_dokument(self, key: str, value: Document):
        """Store Document data in Redis cache
        
        Only caches documents that were successfully downloaded and processed
        """
        if self.disabled:
            return True
        # Skip caching if document wasn't successfully processed
        if not getattr(value, 'download_success', True) or not getattr(value, 'extraction_success', True):
            logger.warning(f"Not caching document {key} due to failed processing")
            return False
            
        try:
            serialized = value.to_json()
            logger.debug(f"Storing dokument {key} in redis")
            success = self.redis_client.set(
                f"dok:{key}", 
                serialized,
                timedelta(minutes=self.cache_expiry_minutes)
            )
            return success
        except Exception as e:
            logger.error(f"Error storing document {key} in cache: {e}")
            return False

    def get_gsvh(self, key: str) -> Optional[models.Vorgang]:
        """Get Vorgang data from cache"""
        if self.disabled:
            return None
        try:
            logger.debug(f"Getting vorgang {key} from cache")
            result = self.redis_client.get(f"vg:{key}")
            
            if not result:
                logger.debug(f"Vorgang {key} not found in cache")
                return None
                
            return models.Vorgang.from_json(result)
        except Exception as e:
            logger.error(f"Error retrieving vorgang {key} from cache: {e}")
            return None

    def get_dokument(self, key: str) -> Optional[Document]:
        """Get Document data from cache"""
        if self.disabled:
            return None
        try:
            logger.debug(f"Getting dokument {key} from cache")
            result = self.redis_client.get(f"dok:{key}")
            
            if not result:
                logger.debug(f"Document {key} not found in cache")
                return None
                
            doc = Document.from_json(result)
            
            # Verify the document was successfully processed
            if not getattr(doc, 'extraction_success', True):
                logger.warning(f"Retrieved document {key} from cache but it was not successfully extracted")
                return None
                
            logger.debug(f"Document {key} retrieved from cache")
            return doc
        except json.JSONDecodeError as e:
            logger.error(f"Error decoding cached document {key}: {e}")
            return None
        except Exception as e:
            logger.error(f"Error retrieving document {key} from cache: {e}")
            return None

    def invalidate_document(self, key: str) -> bool:
        """Remove a specific document from the cache"""
        if self.disabled:
            return True
        try:
            return bool(self.redis_client.delete(f"dok:{key}"))
        except Exception as e:
            logger.error(f"Error invalidating document {key}: {e}")
            return False

    def invalidate_vorgang(self, key: str) -> bool:
        """Remove a specific vorgang from the cache"""
        if self.disabled:
            return True
        try:
            return bool(self.redis_client.delete(f"vg:{key}"))
        except Exception as e:
            logger.error(f"Error invalidating vorgang {key}: {e}")
            return False

    def clear(self):
        """Clear all cache data"""
        if self.disabled:
            return True
        try:
            self.redis_client.flushall()
            logger.info("Cache cleared")
            return True
        except Exception as e:
            logger.error(f"Error clearing cache: {e}")
            return False
            
    def get_cache_stats(self) -> Dict[str, Any]:
        """Get statistics about the cache contents"""
        if self.disabled:
            return {
                'document_count': -1,
                'vorgang_count': -1,
                'total_keys': -1,
                'memory_used': 'unknown'
            }
        try:
            # Get all keys
            all_keys = self.redis_client.keys('*')
            
            # Count document and vorgang keys
            dok_count = len([k for k in all_keys if k.startswith('dok:')])
            vg_count = len([k for k in all_keys if k.startswith('vg:')])
            
            # Get memory info
            memory_info = self.redis_client.info('memory')
            
            return {
                'document_count': dok_count,
                'vorgang_count': vg_count,
                'total_keys': len(all_keys),
                'memory_used': memory_info.get('used_memory_human', 'unknown')
            }
        except Exception as e:
            logger.error(f"Error getting cache stats: {e}")
            return {
                'error': str(e),
                'document_count': -1,
                'vorgang_count': -1
            }