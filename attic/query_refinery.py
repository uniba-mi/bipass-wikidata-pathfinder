import os
from csv import DictReader, DictWriter

import fire

from src.constants import (CSV_FIELDNAMES, CSV_FIELDNAMES_VERBOSE,
                           WIKIDATA_QUERIES_FILEPATHS)
from src.wikidata_wrapper import WikidataWrapper


def refine_wikidata_query_file(which_wikidata_query_file):

    # for performance improvements try to retrieve the labels and descriptions of all entities first
    print("Fetching all labels and descriptions...")

    all_entities = set()

    with open(which_wikidata_query_file) as file:
        queryreader = DictReader(file)

        for row in queryreader:

            wikidata_id_a, wikidata_id_b, _ = row.values()

            all_entities.add(wikidata_id_a)
            all_entities.add(wikidata_id_b)

    # the actual refinement begins here
    # initialize WikidataWrapper
    wikidata_wrapper = WikidataWrapper()
    wikidata_wrapper.fetch_labels_and_descriptions(list(all_entities))

    # process Wikidata query file
    print(f"Refining queries in {which_wikidata_query_file}...")
    with open(which_wikidata_query_file) as file:
        queryreader = DictReader(file)

        for row in queryreader:

            wikidata_id_a, wikidata_id_b, trec_id = row.values()

            print(f"*******************************")
            print(f"Refining query {wikidata_id_a} <and> {wikidata_id_b}...")
             
            # get label and description for each entity
            label_a, description_a = wikidata_wrapper.get_label_and_description(
                wikidata_id_a)
            label_b, description_b = wikidata_wrapper.get_label_and_description(
                wikidata_id_b)

            # filter query if it lacks a description in one or both descriptions
            if not description_a or not description_b:
                print(
                    f"Dropping query because one or both entities lack a description.")
                continue

            # filter query if it uses the label as the description in one or both descriptions
            if label_a == description_a or label_b == description_b:
                print(
                    f"Dropping query because one or both entities use their label as a description.")
                continue

            # filter query if it indicates proprietary "Wikimedia" pages in one or both descriptions
            if "Wikimedia" in description_a or "Wikimedia" in description_b:
                print(
                    f"Dropping query because one or both descriptions indicate proprietary Wikimedia pages.")
                continue

            refined_query = {"wikidata_id_a": wikidata_id_a, "wikidata_id_b": wikidata_id_b, "trec_id": trec_id, "label_a": label_a, "label_b": label_b,
                                      "description_a": description_a, "description_b": description_b}

            yield refined_query


def launch_query_refinery(file_key):

    # get Wikidata query file path
    which_wikidata_query_file = WIKIDATA_QUERIES_FILEPATHS[file_key]

    # prepare output file paths
    output_file_path = which_wikidata_query_file.replace("genre", "refined")
    output_file_path_verbose = which_wikidata_query_file.replace("genre", "refined_verbose")

    if os.path.exists(output_file_path):
        os.remove(output_file_path)

    if os.path.exists(output_file_path_verbose):
        os.remove(output_file_path_verbose)

    # add csv header
    with open(output_file_path, "w+") as output_file:
                querywriter = DictWriter(output_file, fieldnames=CSV_FIELDNAMES)
                querywriter.writeheader()

    with open(output_file_path_verbose, "w+") as output_file:
                querywriter = DictWriter(output_file, fieldnames=CSV_FIELDNAMES_VERBOSE)
                querywriter.writeheader()

    # persist refined queries
    for refined_query in refine_wikidata_query_file(which_wikidata_query_file):

        # concise format
        with open(output_file_path, "a") as output_file:
            querywriter = DictWriter(
                output_file, fieldnames=CSV_FIELDNAMES)
            querywriter.writerow({k: v for k, v in refined_query.items() if k in CSV_FIELDNAMES})

        # verbose format
        with open(output_file_path_verbose, "a") as output_file:
            querywriter = DictWriter(
                output_file, fieldnames=CSV_FIELDNAMES_VERBOSE)
            querywriter.writerow(refined_query)


if __name__ == "__main__":
    fire.Fire()
