from litellm import acompletion
import logging
from typing import Optional

logger = logging.getLogger(__name__)

class LLMConnector:
    def __init__(self, model_name: str, api_key: Optional[str] = None, api_base: Optional[str] = None):
        self.model_name = model_name
        self.api_key = api_key
        self.api_base = api_base

    @classmethod
    def from_openai(cls, api_key: str, model: str = "gpt-4o-mini"):
        return cls(model_name=model, api_key=api_key)

    async def generate(self, prompt: str, text: str) -> str:
        try:
            response = await acompletion(
                model=self.model_name,
                messages=[
                    {"role": "system", "content": "You are a helpful assistant that extracts structured information from documents."},
                    {"role": "user", "content": f"{prompt}\n\n{text}"}
                ],
                api_key=self.api_key,
                api_base=self.api_base,
                temperature=0.0
            )
            return response.choices[0].message.content
        except Exception as e:
            logger.error(f"Error generating response: {e}")
            raise e