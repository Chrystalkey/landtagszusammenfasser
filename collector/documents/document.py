from abc import ABC

class Document(ABC):
    titel = None
    kurztitel=None
    zusammenfassung=None
    volltext=None
    schlagworte=[]
    autoren=[]
    autorpersonen=[]
    hash=None
    last_mod=None
    typ=None
    api_id=None
    link=None
    drucks_nr=None
    meinung=None

    def __init__(self):
        pass

    def download(self):
        pass
    def text_extraction(self):
        pass
    def semantic_extraction(self):
        pass