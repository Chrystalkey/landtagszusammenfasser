class LLMConnector:

    """
    This module will handle all requests going to the large language model (LLM) doing the text processing.
    
    Attributes:
    -----------
    _active : bool
        Specifies if the connection is currently active.
    """

    def __init__(self):
        self.active = False

    def connect(self) -> bool:
        """
        Connects to the database and performs authentication to allow for writes.
        """
        # TODO
        return False

    def close(self):
        """
        Closes the database connection.
        """
        self._active = False

    def perfom_request_about_gesetz(self, text : str) -> str:
        """
        PLACEHOLDER
        Adds the given text to a pre-written prompt.

        Parameters:
        -----------
        test : str
            The text added to the prompt.
        """
        return ""
