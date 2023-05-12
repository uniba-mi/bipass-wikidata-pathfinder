#!/usr/bin/python3

import itertools
import os
import re
from csv import DictReader, DictWriter
import requests
import json

import fire
from genre.entity_linking import \
    get_end_to_end_prefix_allowed_tokens_fn_hf as get_prefix_allowed_tokens_fn
from genre.hf_model import GENRE

from src.constants import (CSV_FIELDNAMES, TREC_QUERIES_FILEPATHS,
                           WIKIDATA_QUERIES_FILEPATHS, WIKIDATA_API, MATCH_MAPPING_PATH)

# load label qid mapping if already available
if os.path.exists(MATCH_MAPPING_PATH):
    with open(MATCH_MAPPING_PATH) as fp:
        match_string_qid_mapping = json.load(fp)
else:
    match_string_qid_mapping = dict()
    

def detect_decoding_errors_line(l, _s):
    """Return decoding errors in a line of text

    Works with text lines decoded with the surrogateescape
    error handler.

    Returns a list of (pos, byte) tuples

    """
    # DC80 - DCFF encode bad bytes 80-FF
    return [(m.start(), bytes([ord(m.group()) - 0xDC00]))
            for m in _s(l)]


def fetch_wikidata_id(entity_label):
    if entity_label in match_string_qid_mapping:
        id = match_string_qid_mapping[entity_label]
    else:
        try:
            id = requests.get(f"{WIKIDATA_API}/id?label={entity_label}").text
        except:
            id = ""

        match_string_qid_mapping[entity_label] = id

        with open(MATCH_MAPPING_PATH, "w+") as fp:
            json.dump(match_string_qid_mapping, fp, indent=3)

    return id


def process_trec_query_file(which_trec_query_file, last_processed_query_id):

    # load huggingface/transformers model
    model = GENRE.from_pretrained(
        "./models/hf_e2e_entity_linking_wiki_abs").eval()

    # initialize surrogates for handling decoding errors
    _surrogates = re.compile(r"[\uDC80-\uDCFF]")

    # process TREC query file
    print(f"Finding queries in {which_trec_query_file}...")
    with open(which_trec_query_file, encoding="utf8", errors="surrogateescape") as raw_query_file:

        for line_counter, line in enumerate(raw_query_file):

            # handle decoding errors present in TREC files
            decoding_errors = detect_decoding_errors_line(
                line, _surrogates.finditer)
            if decoding_errors:
                print(f"Found decoding errors on line {line_counter}:")
                for (col, b) in decoding_errors:
                    print(f" {col + 1:2d}: {b[0]:02x}")
                continue

            # parse TREC query
            line_components = line.split(":")
            trec_id, content = line_components[0], line_components[-1]

            print(f"*******************************")
            print(f"Processing TREC query {trec_id}...")

            # skip query if already processed in previous run
            if int(trec_id) <= int(last_processed_query_id):
                print(
                    f"Query with TREC ID {trec_id} has already been processed -> Skipping...")
                continue

            # skip query if application of GENRE fails
            try:
                # content has to be passed as a list.
                prefix_allowed_tokens_fn = get_prefix_allowed_tokens_fn(
                    model, [content])

                result = model.sample(
                    [content],
                    prefix_allowed_tokens_fn=prefix_allowed_tokens_fn,
                )

            except KeyboardInterrupt:
                exit()
            except:
                print("A problem occurred with GENRE -> Skipping...")
                continue

            result_text = result[0][0]["text"]
            print(f"GENRE result: {result_text}")

            matches = re.findall(r"(?<=\[\s)[\w|\s]*(?=\s\])", result_text)

            # skip query if less than two entities were matched
            if len(matches) >= 2:

                entities = [fetch_wikidata_id(match) for match in matches]
                entities = list(filter(lambda e: e, entities))

                print("Matched QIDs: " +
                      ", ".join(f"{match_string} ({entity})" for entity, match_string in zip(entities, matches)))

            else:
                print("Less than two GENRE matches -> Skipping...")
                continue

            # skip query if less than two Wikidata IDs were identified
            if len(entities) >= 2:
                print("Resulting queries:")

                entity_pairs = itertools.combinations(entities, 2)

                # construct queries for each QID pair
                for entity_a, entity_b in entity_pairs:
                    print(f"{entity_a} <and> {entity_b}")
                    yield {"wikidata_id_a": entity_a, "wikidata_id_b": entity_b, "trec_id": trec_id}
            
            else:
                print("Less than two Wikidata IDs identified -> Skipping...")
                continue



def launch_query_factory(file_key):

    # get TREC query file path
    which_trec_query_file = TREC_QUERIES_FILEPATHS[file_key]
    which_wikidata_query_file = WIKIDATA_QUERIES_FILEPATHS[file_key]

    # create wikidata queries file if not available and get progress
    if not os.path.exists(which_wikidata_query_file):
        with open(which_wikidata_query_file, "w+") as output_file:
            querywriter = DictWriter(output_file, fieldnames=CSV_FIELDNAMES)
            querywriter.writeheader()
            last_processed_query_id = 0
    else:
        with open(which_wikidata_query_file, "r") as output_file:
            queryreader = DictReader(output_file)
            last_processed_query_id = 0

            for row in queryreader:
                last_processed_query_id = row["trec_id"]

    # persist generated queries
    for query in process_trec_query_file(which_trec_query_file, last_processed_query_id):
        with open(which_wikidata_query_file, "a") as output_file:
            querywriter = DictWriter(
                output_file, fieldnames=CSV_FIELDNAMES)
            querywriter.writerow(query)


if __name__ == "__main__":
    fire.Fire()
