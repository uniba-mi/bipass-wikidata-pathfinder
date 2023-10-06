#!/usr/bin/env python3

from time import sleep
from string import Template
from flask import Flask, jsonify, request
from SPARQLWrapper import SPARQLWrapper, JSON


app = Flask(__name__)
endpoint_url = "https://query.wikidata.org/sparql"
sparql_wrapper = SPARQLWrapper(
    endpoint_url, agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/50.0.2661.102 Safari/537.36")


def query_wikidata(query):
    sparql_wrapper.setQuery(query)
    sparql_wrapper.setReturnFormat(JSON)

    try:
        results = sparql_wrapper.query().convert()
        sleep(3) # required to comply with Wikidata's rate limit policy
    except Exception as e:
        print(f"An error occurred while querying Wikidata: {e}")
        results = {}

    return results


@app.route("/")
def root():
    return "Hello from the Wikidata API."


@app.route("/adjacent_entities", methods=["GET"])
def adjacent_entities():

    def make_request(depth):
        select_block_content = " ".join(
            [f'?subject_id{ctr} ?the_subject_label{ctr} ?the_subject_description{ctr} ?predicate_id{ctr} ?the_predicate_label{ctr} ?object_id{ctr} ?the_object_label{ctr} ?the_object_description{ctr}' for ctr in range(depth)])
        select_block = f"SELECT {select_block_content}"

        where_block_template = Template("""
            $subject_id $predicate_id $object_id .

            FILTER ( CONTAINS( str($subject_id), "http://www.wikidata.org/entity/Q" ) ) .
            FILTER ( CONTAINS( str($predicate_id), "http://www.wikidata.org/prop/direct/P" ) ) .
            FILTER ( CONTAINS( str($object_id), "http://www.wikidata.org/entity/Q" ) ) .

            $subject_id rdfs:label $the_subject_label .
            FILTER ( lang($the_subject_label) = "en" ) .

            $subject_id schema:description $the_subject_description .
            FILTER ( lang($the_subject_description) = "en" ) .

            $object_id rdfs:label $the_object_label .
            FILTER ( lang($the_object_label) = "en" ) .
            FILTER ( !CONTAINS ( $the_object_label, "Wiki" ) ) .
            FILTER REGEX ( $the_object_label, "^[A-z0-9 -]+$$" ) .

            $object_id schema:description $the_object_description .
            FILTER ( lang($the_object_description) = "en" ) .
            FILTER ( !CONTAINS ( $the_object_description, "Wiki" ) ) .

            $foo wikibase:directClaim $predicate_id .
            $foo rdfs:label $the_predicate_label.
            FILTER ( lang($the_predicate_label) = "en" ) .
            FILTER ( !CONTAINS ( $the_predicate_label, "Wiki" ) ) .
            FILTER ( $predicate_id != wdt:P1343 ) .""")

        initial_where_block_content = where_block_template.substitute(
            subject_id="$subject_id0",
            the_subject_label="$the_subject_label0",
            the_subject_description="$the_subject_description0",
            predicate_id="$predicate_id0",
            the_predicate_label="$the_predicate_label0",
            object_id="$object_id0",
            the_object_label="$the_object_label0",
            the_object_description="$the_object_description0",
            foo="$foo0")

        other_where_block_content = " ".join([where_block_template.substitute(
            subject_id=f"$object_id{ctr-1}",
            the_subject_label=f"$the_subject_label{ctr}",
            the_subject_description=f"$the_subject_description{ctr}",
            predicate_id=f"$predicate_id{ctr}",
            the_predicate_label=f"$the_predicate_label{ctr}",
            object_id=f"$object_id{ctr}",
            the_object_label=f"$the_object_label{ctr}",
            the_object_description=f"$the_object_description{ctr}",
            foo=f"$foo{ctr}") for ctr in range(1, depth)])

        where_block = f"""WHERE {{
            VALUES ?subject_id0 {{ <http://www.wikidata.org/entity/{entity}> }}
            {initial_where_block_content}
            {other_where_block_content}
            }}"""

        query = f"""{select_block}
            {where_block}"""

        return query_wikidata(query)

    entity = request.args.get("entity")
    depth = int(request.args.get("depth"))

    results = make_request(depth)

    if not results:
        return jsonify(
            {
                "adjacent_entities": {entity: {}},
                "q_labels": {},
                "q_descriptions": {},
                "p_labels": {},
            }
        ), 200

    clean_adjacent_entities = {entity: []}
    clean_q_labels = dict()
    clean_q_descriptions = dict()
    clean_p_labels = dict()
    default = []

    for result in results["results"]["bindings"]:
        init_subject = result["subject_id0"]["value"].split("/")[-1]
        init_predicate = result["predicate_id0"]["value"].split("/")[-1]
        init_object = result["object_id0"]["value"].split("/")[-1]

        clean_adjacent_entities.setdefault(
            init_subject, default).append(f"{init_predicate}-{init_object}")
        clean_adjacent_entities[init_subject] = list(
            set(clean_adjacent_entities[init_subject]))

        clean_q_labels[init_subject] = result["the_subject_label0"]["value"]
        clean_q_descriptions[init_subject] = result["the_subject_description0"]["value"]
        clean_q_labels[init_object] = result["the_object_label0"]["value"]
        clean_q_descriptions[init_object] = result["the_object_description0"]["value"]

        clean_p_labels[result["predicate_id0"]["value"].split(
            "/")[-1]] = result["the_predicate_label0"]["value"]

        for ctr in range(1, depth):
            subject = result[f"object_id{ctr-1}"]["value"].split("/")[-1]
            predicate = result[f"predicate_id{ctr}"]["value"].split("/")[-1]
            object = result[f"object_id{ctr}"]["value"].split("/")[-1]

            clean_adjacent_entities.setdefault(
                subject, default).append(f"{predicate}-{object}")
            clean_adjacent_entities[subject] = list(
                set(clean_adjacent_entities[subject]))

            clean_q_labels[object] = result[f"the_object_label{ctr}"]["value"]
            clean_q_descriptions[object] = result[f"the_object_description{ctr}"]["value"]
            clean_p_labels[result[f"predicate_id{ctr}"]["value"].split(
                "/")[-1]] = result[f"the_predicate_label{ctr}"]["value"]


    return jsonify(
        {
            "adjacent_entities": clean_adjacent_entities,
            "q_labels": clean_q_labels,
            "q_descriptions": clean_q_descriptions,
            "p_labels": clean_p_labels,
        }
    ), 200


@app.route("/label_description", methods=["GET"])
def label_description():
    entity = request.args.get("entity")

    query = f"""
    SELECT ?subject_id ?the_subject_label ?the_subject_description
    WHERE {{     
    VALUES ?subject_id {{ <http://www.wikidata.org/entity/{entity}> }}       

    $subject_id rdfs:label $the_subject_label .       
    FILTER ( lang($the_subject_label) = "en" ) .
    
    $subject_id schema:description $the_subject_description .
    FILTER ( lang($the_subject_description) = "en" ) .
    }}
    """

    results = query_wikidata(query)

    if not results:
        return jsonify(
            {
                "label": "",
                "description": "",
            }
        ), 200
    
    result_bindings = results["results"]["bindings"]

    label = result_bindings[0]["the_subject_label"]["value"].split(
    "/")[-1]

    description = result_bindings[0]["the_subject_description"]["value"].split(
    "/")[-1]
        
    return jsonify(
        {
            "label": label,
            "description": description,
        }
    ), 200


@app.route("/id", methods=["GET"])
def id():
    label = request.args.get("label")

    query = f"""SELECT DISTINCT ?item
        WHERE
        {{
        ?item ?label "{label}".  
        ?article schema:about ?item .
        ?article schema:inLanguage "en" .
        ?article schema:isPartOf <https://en.wikipedia.org/>.	
        SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
        }}"""

    results = query_wikidata(query)
    
    if not results:
        return jsonify(
            {
                "id": "",
            }
        ), 200

    result_bindings = results["results"]["bindings"]

    id = result_bindings[0]["item"]["value"].split(
        "/")[-1] if result_bindings else ""
    
    return id


if __name__ == "__main__":
    app.run(debug=True, host="0.0.0.0")
