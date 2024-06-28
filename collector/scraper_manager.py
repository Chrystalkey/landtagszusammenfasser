from database_connector import DatabaseConnector
from llm_collector import LLMCollector

def main():
    print ("Starting collector manager.")
    db_connector = DatabaseConnector()
    llm_collector = LLMCollector()


if __name__ == "__main__":
    main()
