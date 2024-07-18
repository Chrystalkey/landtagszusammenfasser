class CUPGesVH:
    uuid = None
    titel = None
    off_titel = None
    url_gesetzesblatt = None
    verf_aendernd = False
    trojaner = False
    federfuehrung = None
    initiator = None
    schlagworte = []
    status = None
    eigenschaften = []
    parlamente = []



class DatabaseConnector:

    """
    This module will handle all requests going to the database so that the scrapers are not required to re-implement this connection by themselves.
    
    Attributes:
    -----------
    _active : bool
        Specifies if the connection is currently active.
    """

    def __init__(self):
        self._connection = None
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

    def write_new_gesetzesvorhaben(self, id : int, text : str) -> bool:
        """
        PLACEHOLDER
        Send an information update to the database about a new Gesetzesvorhaben.
        This should only be allowed by the scraper manager.

        Parameters:
        -----------
        id : int
            The id of the new Gesetzesvorhaben.
        test : str
            The text of the new Gesetzesvorhaben.
        """
        return True
