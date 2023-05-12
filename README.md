# Wikidata Pathfinder

<p align="center">
    <img src="https://img.shields.io/badge/license-GPLv3-green.svg" alt="license">
    <br>
</p>

<p align="center">
    <a href="#requirements">Requirements</a>
    •
    <a href="#wikidata-query-factory">Wikidata Query Factory</a>
    •
    <a href="#wikidata-pathfinder">Wikidata pathfinder</a>
    •
    <a href="#license">License</a>
</p>

## Requirements

Only `docker` and `docker-compose` are required to run this project's components. All dependencies are installed using the corresponding Dockerfiles. The three components within this project are:

 - Wikidata Query Factory
 - Wikidata Pathfinder
 - End-to-end Search Interface (Not yet available, WIP)

The following paragraphs will describe the purpose and the usage of these components.

## Wikidata Query Factory

The purpose of this component is to derive dual-entity queries for pathfinding in Wikidata from the [TREC 2007 Million Queries Track dataset]{http://trec.nist.gov/data/million.query07.html}. For identifying and disambiguating the entities mentioned in the TREC queries the [GENRE entity linker](https://github.com/facebookresearch/GENRE) is employed in the query factory script. Other scripts for refining and calculating statistics on the derived query datasets are also provided.

The repository already contains the derived query dataset but the query factory can be rerun, of course.

### Query Datasets

The derived dual-entity query dataset can be found [here](./data/wikidata_queries_10000_topics_genre.csv) in CSV format. The columns have the following meaning:

- wikidata_id_a: The Wikidata ID of the first entity of the query
- wikidata_id_b: The Wikidata ID of the second entity of the query
- trec_id: The ID of the original TREC query

### Usage

To rerun the query factory proceed as follows:

1. Select the TREC file from which queries should be derived by adjusting the commented parts in the [query_factory.py](./src/query_factory.py). 
2. Run ``docker compose run query_factory`` from the root directory.
3. In the new bash run ``factory 07`` to start the query factory.

## Wikidata Pathfinder

1. Launch the Wikidata API via `docker-compose run --service-ports wikidata_api` in a separate bash.
2. Launch the Wembed API via `docker-compose run --service-ports wembed_api` in a separate bash.
3. Run `docker-compose run pathfinder` in a separate bash to get a bash in the main component of the Pathfinder. There are several commands that can be used in this new bash:
    1. Run `cargo run -- playground` to launch the pathfinder on a few example queries.
    2. Run `cargo run -- optimizer` to run the optimizer for fitting the search parameters alpha, beta, and gamma. Warning: This will overwrite the optimizer result in the [optimizer results file](./data/optimizer_results.csv). The optimizer results file contains one additional line that repeats the configuration with the best result.
    3. Run `cargo run -- benchmark` to run the benchmark. Warning: This will overwrite the benchmark result already present in the [benchmark results files](./data/).

## License

See [LICENSE](./LICENSE)
