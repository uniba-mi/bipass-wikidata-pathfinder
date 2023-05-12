from collections import defaultdict, Counter
import statistics
import plotext
from csv import DictReader
from constants import WIKIDATA_QUERIES_FILEPATHS


def count_queries(file_path):
    with open(file_path) as file:
        return len(file.readlines())


def count_entity_occurrences(file_path):
    entity_occurrences = defaultdict(lambda: 0)

    with open(file_path) as file:
        queryreader = DictReader(file)

        for row in queryreader:

            entity_occurrences[row["wikidata_id_a"]] += 1
            entity_occurrences[row["wikidata_id_b"]] += 1

    entity_occurrences = dict(
        sorted(entity_occurrences.items(), key=lambda item: item[1]))

    return entity_occurrences


for file_path in WIKIDATA_QUERIES_FILEPATHS:
    print(f"*******************************")
    print(f"Processing {file_path}...")

    # query count
    number_of_queries = count_queries(file_path)
    print(f"{number_of_queries} queries found.")

    # entity occurrence distribution
    entity_occurrences = count_entity_occurrences(file_path)
    occurrences = list(entity_occurrences.values())
    print(f"QID occurrence distribution: {min(occurrences)} min, {max(occurrences)} max, {statistics.mean(occurrences)} average, {statistics.median(occurrences)} median")

    plotext.simple_bar(Counter(occurrences).keys(), Counter(occurrences).values(), width = 80)
    plotext.show()

    plotext.clear_figure()
