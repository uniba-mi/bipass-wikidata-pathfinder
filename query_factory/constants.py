TREC_QUERIES_FILEPATHS = {
    "07": "./trec/07-million-query-topics.1-10000",
    "08": "./trec/08.million-query-topics.10001-20000",
    "09": "./trec/09.mq.topics.20001-60000"
}

WIKIDATA_QUERIES_FILEPATHS = {
    "07": "../data/wikidata_queries_10000_topics_genre.csv",
    "08": "../data/wikidata_queries_20000_topics_genre.csv",
    "09": "../data/wikidata_queries_60000_topics_genre.csv"
}

CSV_FIELDNAMES = ["wikidata_id_a", "wikidata_id_b", "trec_id"]

WIKIDATA_API = "http://127.0.0.1:5000"

MATCH_MAPPING_PATH = "../data/match_string_qid_mapping.json"
