from SPARQLWrapper import SPARQLWrapper, JSON
import json
from time import sleep
import os


class WikidataWrapper():

    def __init__(self, sparql_endpoint="https://query.wikidata.org/sparql", api_endpoint="https://www.wikidata.org/wiki/Special:EntityData/"):
        # initialize endpoints
        self.sparql_wrapper = SPARQLWrapper(
            sparql_endpoint, agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/50.0.2661.102 Safari/537.36")

        self.api_endpoint = api_endpoint

        # load label qid mapping if already available
        if os.path.exists("./data/match_string_qid_mapping.json"):
            with open("./data/match_string_qid_mapping.json") as fp:
                self.match_string_qid_mapping = json.load(fp)
        else:
            self.match_string_qid_mapping = dict()

        # load label qid mapping if already available
        if os.path.exists("./data/qid_label_mapping.json"):
            with open("./data/qid_label_mapping.json") as fp:
                self.qid_label_mapping = json.load(fp)
        else:
            self.qid_label_mapping = dict()

        # load qid description mapping if already available
        if os.path.exists("./data/qid_description_mapping.json"):
            with open("./data/qid_description_mapping.json") as fp:
                self.qid_description_mapping = json.load(fp)
        else:
            self.qid_description_mapping = dict()

        # load qid outgoing mapping if already available
        if os.path.exists("./data/qid_outgoing_mapping.json"):
            with open("./data/qid_outgoing_mapping.json") as fp:
                self.qid_outgoing_mapping = json.load(fp)
        else:
            self.qid_outgoing_mapping = dict()

    def __query_wikidata__(self, query):
        self.sparql_wrapper.setQuery(query)
        self.sparql_wrapper.setReturnFormat(JSON)

        try:
            results = self.sparql_wrapper.query().convert()
            result_bindings = results["results"]["bindings"]
        except Exception as e:
            print(e)
            result_bindings = []

        sleep(3)

        return result_bindings

    def get_wikidata_id(self, entity_label):

        if entity_label in self.match_string_qid_mapping:
            qid = self.match_string_qid_mapping[entity_label]
        else:
            query = f"""SELECT DISTINCT ?item
                WHERE
                {{ 
                {{
                    ?item ?label "{entity_label}"@en.  
                    ?article schema:about ?item .
                    ?article schema:inLanguage "en" .
                    ?article schema:isPartOf <https://en.wikipedia.org/>.	
                    SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
                }}
                UNION
                {{
                    ?item ?label "{entity_label}".  
                    ?article schema:about ?item .
                    ?article schema:inLanguage "en" .
                    ?article schema:isPartOf <https://en.wikipedia.org/>.	
                    SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
                }}
                UNION
                {{
                    ?item ?label "{entity_label.lower()}"@en.  
                    ?article schema:about ?item .
                    ?article schema:inLanguage "en" .
                    ?article schema:isPartOf <https://en.wikipedia.org/>.	
                    SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
                }}
                UNION
                {{
                    ?item ?label "{entity_label.lower()}".  
                    ?article schema:about ?item .
                    ?article schema:inLanguage "en" .
                    ?article schema:isPartOf <https://en.wikipedia.org/>.	
                    SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
                }}
                UNION
                {{
                    ?item ?label "{" ".join([word.capitalize() for word in entity_label.split()])}"@en.  
                    ?article schema:about ?item .
                    ?article schema:inLanguage "en" .
                    ?article schema:isPartOf <https://en.wikipedia.org/>.	
                    SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
                }}
                UNION
                {{
                    ?item ?label "{" ".join([word.capitalize() for word in entity_label.split()])}".  
                    ?article schema:about ?item .
                    ?article schema:inLanguage "en" .
                    ?article schema:isPartOf <https://en.wikipedia.org/>.	
                    SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
                }}
                }}"""

            result_bindings = self.__query_wikidata__(query)
            qid = result_bindings[0]["item"]["value"].split(
                "/")[-1] if len(result_bindings) else ""

            self.match_string_qid_mapping[entity_label] = qid

            with open("./data/match_string_qid_mapping.json", "w+") as fp:
                json.dump(self.match_string_qid_mapping, fp, indent=3)

        return qid

    def get_adjacent_entities(self, qid):

        if qid in self.qid_outgoing_mapping:
            adjacent_entities = self.qid_outgoing_mapping[qid]
        else:
            query = f"""
                SELECT ?subject_id ?object_id

                WHERE {{
                VALUES ?subject_id {{ <http://www.wikidata.org/entity/{qid}> }}

                $subject_id $predicate_id $object_id .

                FILTER ( CONTAINS( str($subject_id), "http://www.wikidata.org/entity/Q" ) ) .
                FILTER ( CONTAINS( str($predicate_id), "http://www.wikidata.org/prop/direct/P" ) ) .
                FILTER ( CONTAINS( str($object_id), "http://www.wikidata.org/entity/Q" ) ) .

                $subject_id rdfs:label $the_subject_label .
                FILTER ( lang($the_subject_label) = "en" ) .
                FILTER regex ( $the_subject_label, "^[A-z0-9 -]+$$" ) .

                $foo wikibase:directClaim $predicate_id .
                $foo rdfs:label $the_predicate_label.
                FILTER ( lang($the_predicate_label) = "en" ) .
                FILTER ( $predicate_id != wdt:P1343 ) .
                }}
                """

            result_bindings = self.__query_wikidata__(query)
            adjacent_entities = [result["object_id"]["value"].split(
                "/")[-1] for result in result_bindings]

            self.qid_outgoing_mapping[qid] = adjacent_entities

            with open("./data/qid_outgoing_mapping.json", "w+") as fp:
                json.dump(self.qid_outgoing_mapping, fp, indent=3)

        return adjacent_entities

    def fetch_labels_and_descriptions(self, qid_list):

        def make_fetch(partial_qid_list):
            select_part = "SELECT ?entity_id ?entity_label ?entity_description"

            query_blocks = []

            for qid in partial_qid_list:
                query_block = f"""
                    {{
                        VALUES ?entity_id {{ <http://www.wikidata.org/entity/{qid}> }}

                        $entity_id rdfs:label $entity_label .
                        FILTER ( lang($entity_label) = "en" ) .
                        FILTER regex ( $entity_label, "^[A-z0-9 -]+$$" ) .

                        $entity_id schema:description $entity_description .
                        FILTER ( lang($entity_description) = "en" ) .
                    }}
                """

                query_blocks.append(query_block)

            query_body = " UNION ". join(query_blocks)

            query = f"""
                {select_part}
                WHERE {{
                    {query_body}
                }}
            """

            compressed_query = " ".join(query.split())

            self.sparql_wrapper.setQuery(compressed_query)
            self.sparql_wrapper.setReturnFormat(JSON)

            result_bindings = self.__query_wikidata__(query)

            for result in result_bindings:

                entity_id = result["entity_id"]["value"] if "entity_id" in result else ""
                entity_label = result["entity_label"]["value"] if "entity_label" in result else ""
                entity_description = result["entity_description"]["value"] if "entity_description" in result else ""

                entity = entity_id.split("/")[-1]

                self.qid_label_mapping[entity] = entity_label
                self.qid_description_mapping[entity] = entity_description

            with open("./data/qid_label_mapping.json", "w+") as fp:
                json.dump(self.qid_label_mapping, fp, indent=3)

            with open("./data/qid_description_mapping.json", "w+") as fp:
                json.dump(self.qid_description_mapping, fp, indent=3)

        qid_list = list(set(filter(
            lambda qid: qid not in self.qid_label_mapping or qid not in self.qid_description_mapping, qid_list)))
        chunk_size = 10
        qid_list_chunks = [qid_list[i:i + chunk_size]
                           for i in range(0, len(qid_list), chunk_size)]

        for chunk in qid_list_chunks:
            print(f"Fetching chunk with Wikidata IDs: {', '.join(chunk)}")
            make_fetch(chunk)

    def get_label_and_description(self, qid):
        label, description = "", ""

        if qid in self.qid_label_mapping:
            label = self.qid_label_mapping[qid]

        if qid in self.qid_description_mapping:
            description = self.qid_description_mapping[qid]

        return label, description


if __name__ == "__main__":
    wikidata_wrapper = WikidataWrapper()
    wikidata_wrapper.fetch_labels_and_descriptions(["Q42", "Q12125", "Q1221"])
