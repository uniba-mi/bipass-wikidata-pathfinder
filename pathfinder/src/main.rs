use env_logger;
use log::info;
use simplers_optimization::Optimizer;
use statrs::statistics::Statistics;
use std::env;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use toml::Table;

#[path = "./pathfinder.rs"]
mod pathfinder;

#[path = "./store_connector.rs"]
mod store_connector;
use crate::store_connector::StoreConnector;

#[path = "./api_connector.rs"]
mod api_connector;
use crate::api_connector::ApiConnector;

fn main() {
    // initialize logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    // load configuration
    let config_data: std::string::String =
        fs::read_to_string("./config.toml").expect("config.toml could not be read.");
    let config: toml::map::Map<std::string::String, toml::Value> =
        config_data.parse::<Table>().unwrap();

    // read command line arguments
    let args: Vec<String> = env::args().collect();
    let option = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| String::from("playground"));

    // run corresponding function
    match option.as_str() {
        "playground" => playground(&config),
        "optimizer" => optimizer(&config),
        "benchmark" => benchmark(&config),
        _ => panic!("This command line option is not supported."),
    }
}

// TODO maybe switch to gradient descent or so with argmin crate (https://github.com/argmin-rs/argmin)
fn optimizer(config: &toml::map::Map<String, toml::Value>) {
    // create ApiConnector instance
    let api_connector: ApiConnector = api_connector::ApiConnector::new(
        String::from(config["wembed_api"].as_str().unwrap()),
        String::from(config["wikidata_api"].as_str().unwrap()),
    );

    // create StoreConnector instance
    let store_connector: StoreConnector = store_connector::StoreConnector::new(
        &api_connector,
        String::from(config["label_mapping_path"].as_str().unwrap()),
        String::from(config["desc_mapping_path"].as_str().unwrap()),
        String::from(config["distance_mapping_path"].as_str().unwrap()),
        String::from(config["adjacency_list_path"].as_str().unwrap()),
    );

    // create Pathfinder instance
    let pathfinder = pathfinder::Pathfinder::new(
        &store_connector,
        String::from(config["distance_method"].as_str().unwrap()),
        config["entity_limit"].as_integer().unwrap() as usize,
    );

    // collect sample queries for the optimization from the Wikidata query files
    let query_file_paths = config["query_file_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect::<Vec<String>>();

    let mut some_queries: Vec<(String, String, String)> = vec![];
    let sample_percentage = config["optimizer_sample_percentage"].as_float().unwrap();

    // parse individual query files
    for path in query_file_paths {
        let mut reader = csv::Reader::from_path(path).unwrap();

        // read content
        let new_queries: Vec<(String, String, String)> = reader
            .records()
            .map(|r| r.unwrap())
            .map(|r| {
                (
                    r.get(0).unwrap().to_owned(),
                    r.get(1).unwrap().to_owned(),
                    r.get(2).unwrap().to_owned(),
                )
            })
            .collect();

        // number of queries to be collected from this set
        let query_number = (sample_percentage * new_queries.len() as f64) as usize;

        // collect and add queries
        some_queries.extend_from_slice(&new_queries[..query_number]);
    }

    info!(
        "Collected {} queries for the optimization.",
        some_queries.len()
    );

    // create or clear file for results
    let mut file = File::create(config["optimizer_results_path"].as_str().unwrap()).unwrap();
    writeln!(file, "alpha,beta,gamma,objective_value").unwrap();

    // the function to be optimized
    let f = |search_params: &[f64]| {
        // to collect the scores of the individual pathfinder runs
        let mut collected_scores: Vec<f64> = Vec::new();

        // iterate sample queries
        for query in &some_queries {
            let (source_entity, target_entity, trec_id) = query;

            info!(
                "******* Optimizer processes query from TREC {} query with alpha={}, beta={}, gamma={}",
                trec_id, search_params[0], search_params[1], search_params[2]
            );

            // find a path given the provided configuration
            let (_, _, score, _) = pathfinder.find_path(
                source_entity,
                target_entity,
                (search_params[0], search_params[1], search_params[2]),
            );

            collected_scores.push(score);
        }

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(config["optimizer_results_path"].as_str().unwrap())
            .unwrap();

        // calculate, store, and return the average number of visited entities
        let objective_value = Statistics::mean(collected_scores);

        writeln!(
            file,
            "{},{},{},{}",
            search_params[0], search_params[1], search_params[2], objective_value
        )
        .unwrap();

        objective_value
    };

    // the intervals for the parameters alpha, beta, and gamma
    let input_interval = vec![(0.0, 1.0), (0.0, 1.0), (0.0, 1.0)];

    // the number of iterations
    let iterations = config["optimizer_iterations"].as_integer().unwrap() as usize;

    // run the optimizer
    let (min_value, coordinates) = Optimizer::minimize(&f, &input_interval, iterations);

    writeln!(
        file,
        "{},{},{},{}",
        coordinates[0], coordinates[1], coordinates[2], min_value
    )
    .unwrap();
}

fn benchmark(config: &toml::map::Map<String, toml::Value>) {
    // create ApiConnector instance
    let api_connector: ApiConnector = api_connector::ApiConnector::new(
        String::from(config["wembed_api"].as_str().unwrap()),
        String::from(config["wikidata_api"].as_str().unwrap()),
    );

    // create StoreConnector instance
    let store_connector: StoreConnector = store_connector::StoreConnector::new(
        &api_connector,
        String::from(config["label_mapping_path"].as_str().unwrap()),
        String::from(config["desc_mapping_path"].as_str().unwrap()),
        String::from(config["distance_mapping_path"].as_str().unwrap()),
        String::from(config["adjacency_list_path"].as_str().unwrap()),
    );

    // create Pathfinder instance
    let pathfinder = pathfinder::Pathfinder::new(
        &store_connector,
        String::from(config["distance_method"].as_str().unwrap()),
        config["entity_limit"].as_integer().unwrap() as usize,
    );

    // read optimized configuration from file
    let path = config["optimizer_results_path"].as_str().unwrap();
    let mut reader = csv::Reader::from_path(path).unwrap();

    let optimizer_results = reader
        .records()
        .map(|r| r.unwrap())
        .map(|r| {
            (
                r.get(0).unwrap().to_owned().parse::<f64>().unwrap(),
                r.get(1).unwrap().to_owned().parse::<f64>().unwrap(),
                r.get(2).unwrap().to_owned().parse::<f64>().unwrap(),
            )
        })
        .collect::<Vec<(f64, f64, f64)>>();

    let optimized_params = optimizer_results.last().unwrap();

    println!("{:?}", optimized_params);

    // create configurations for benchmarking
    let benchmark_configs = vec![
        optimized_params,
        &(0.0, 1.0, 0.0), // breadth-first
        &(1.0, 0.0, 1.0),
        &(0.0, 0.0, 1.0),
        &(1.0, 0.5, 1.0),
    ];

    // collect test queries for the benchmark from the Wikidata query files
    // these are all queries not used for the optimization
    let query_file_paths = config["query_file_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect::<Vec<String>>();

    let mut some_queries: Vec<(String, String, String)> = vec![];
    let sample_percentage = config["optimizer_sample_percentage"].as_float().unwrap();

    // parse individual query files
    for path in query_file_paths {
        let mut reader = csv::Reader::from_path(path).unwrap();

        // read content
        let new_queries: Vec<(String, String, String)> = reader
            .records()
            .map(|r| r.unwrap())
            .map(|r| {
                (
                    r.get(0).unwrap().to_owned(),
                    r.get(1).unwrap().to_owned(),
                    r.get(2).unwrap().to_owned(),
                )
            })
            .collect();

        // number of queries to be collected from this set
        let query_number = (sample_percentage * new_queries.len() as f64) as usize;

        // collect and add queries
        some_queries.extend_from_slice(&new_queries[query_number..]);
    }

    info!(
        "Collected {} queries for the benchmark.",
        some_queries.len()
    );

    // benchmark each config
    for hyperparameter_config in benchmark_configs {
        // create variables for storing benchmark results
        let mut total_successes = 0;
        let mut collected_visited_entities: Vec<usize> = vec![];
        let mut collected_path_lengths: Vec<usize> = vec![];

        // run pathfinder for test queries
        for query in &some_queries {
            let (source_entity, target_entity, trec_id) = query;

            info!(
                "******* Benchmarking TREC {} query with {}, {}, {}",
                trec_id, hyperparameter_config.0, hyperparameter_config.1, hyperparameter_config.2
            );

            // execute the pathfinding
            let (found_path_forwards, found_path_backwards, _, visited_entities) = pathfinder
                .find_path(
                    source_entity,
                    target_entity,
                    (
                        hyperparameter_config.0,
                        hyperparameter_config.1,
                        hyperparameter_config.2,
                    ),
                );

            // update results
            if found_path_forwards.is_empty() {
                collected_visited_entities.push(0);
                collected_path_lengths.push(0);
            } else {
                total_successes += 1;
                collected_visited_entities.push(visited_entities);

                let path_length = if found_path_backwards.is_some() {
                    found_path_forwards.len() + found_path_backwards.unwrap().len() - 2
                } else {
                    found_path_forwards.len() - 1
                };
                collected_path_lengths.push(path_length);
            }
        }

        // calculate success rate
        let success_rate: f32 = total_successes as f32 / some_queries.len() as f32;

        // calculate average visited entities for successful cases
        let visited_entities_cleaned: Vec<f32> = collected_visited_entities
            .iter()
            .map(|n| *n as f32)
            .filter(|n| n.to_owned() > 0.0)
            .collect();

        let average_visited_entities: f32 =
            visited_entities_cleaned.iter().sum::<f32>() / visited_entities_cleaned.len() as f32;

        // calculate average path length for successful cases
        let path_lengths_cleaned: Vec<f32> = collected_path_lengths
            .iter()
            .map(|n| *n as f32)
            .filter(|n| n.to_owned() > 0.0)
            .collect();

        let average_path_lengths: f32 =
            path_lengths_cleaned.iter().sum::<f32>() / path_lengths_cleaned.len() as f32;

        // log stats
        info!(
            "These are the stats for: {}, {}, {}",
            hyperparameter_config.0, hyperparameter_config.1, hyperparameter_config.2
        );

        info!("Success rate: {}", success_rate);

        info!("Average visited entities: {}", average_visited_entities);

        info!("Average path lengths: {}", average_path_lengths);

        // store results
        let result_path = format!(
            "{}_{}_{}_{}.toml",
            config["benchmark_results_path"].as_str().unwrap(),
            hyperparameter_config.0,
            hyperparameter_config.1,
            hyperparameter_config.2
        );

        let mut file = File::create(result_path).unwrap();

        let toml_content = format!(
            "number_of_queries = {}
success_rate = {}
average_visited_entities = {}
path_lengths_entities = {}
",
            some_queries.len(),
            success_rate,
            average_visited_entities,
            average_path_lengths,
        );

        file.write_all(toml_content.as_bytes()).unwrap();
    }
}

fn playground(config: &toml::map::Map<String, toml::Value>) {
    // create ApiConnector instance
    let api_connector: ApiConnector = api_connector::ApiConnector::new(
        String::from(config["wembed_api"].as_str().unwrap()),
        String::from(config["wikidata_api"].as_str().unwrap()),
    );

    // create StoreConnector instance
    let store_connector: StoreConnector = store_connector::StoreConnector::new(
        &api_connector,
        String::from(config["label_mapping_path"].as_str().unwrap()),
        String::from(config["desc_mapping_path"].as_str().unwrap()),
        String::from(config["distance_mapping_path"].as_str().unwrap()),
        String::from(config["adjacency_list_path"].as_str().unwrap()),
    );

    // create Pathfinder instance
    let pathfinder = pathfinder::Pathfinder::new(
        &store_connector,
        String::from(config["distance_method"].as_str().unwrap()),
        config["entity_limit"].as_integer().unwrap() as usize,
    );

    let search_params = (0.23031994047619048, 0.02808779761904762, 0.58984375);

    // source -> target: path length 1
    let mut entity_a = "Q42";
    let mut entity_b = "Q5";

    pathfinder.find_path(entity_a, entity_b, search_params);

    // target -> source: path length 1
    pathfinder.find_path(entity_b, entity_a, search_params);

    // source -> intersecting <- target: path length 2
    entity_a = "Q42";
    entity_b = "Q762";

    pathfinder.find_path(entity_a, entity_b, search_params);

    // target -> intersecting <- source: path length 2
    pathfinder.find_path(entity_b, entity_a, search_params);

    // source -> intersecting <- target: path length 3
    entity_a = "Q42";
    entity_b = "Q389908";

    pathfinder.find_path(entity_a, entity_b, search_params);

    // target -> intersecting <- source: path length 3
    pathfinder.find_path(entity_b, entity_a, search_params);

    // actual test query from derived query set
    entity_a = "Q376657";
    entity_b = "Q1951366";

    pathfinder.find_path(entity_a, entity_b, search_params);
}
