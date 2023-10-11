use crate::store_connector::StoreConnector;

// TODO Improve costs function; maybe take property quality into account?
// Calculates the costs of a path comprising entities.
// Costs mapping fScore from https://en.wikipedia.org/wiki/A*_search_algorithm cannot be used for us as we use the average (!) semantic distance in the g costs
pub fn calculate_costs(
    store_connector: &StoreConnector,
    source_entity: &str,
    target_entity: &str,
    path: Vec<String>,
    hyperparameter_config: (f64, f64, f64),
) -> i64 {
    
    let (alpha, beta, gamma) = hyperparameter_config;
    let (g1, g2, h, costs);

    let directional_target_entity;

    // sets a directional target entity to account for the direction of the path
    if path[0] == source_entity {
        directional_target_entity = target_entity;
    } else if path[0] == target_entity {
        directional_target_entity = source_entity;
    } else {
        panic!("Something is wrong with the candidate paths.");
    }

    // get a slice of all entities except the last entity on the path
    let path_so_far = &path[0..path.len() - 1];

    // calculate average semantic distance of path so far to target entity
    if alpha == 0.0 || path_so_far.is_empty() {
        g1 = 0.0;
    } else {
        let total_distance: f64 = path.iter().fold(0.0, |acc, e| {
            acc + store_connector.get_semantic_distance(e, directional_target_entity)
        });
        
        let average_distance = total_distance / path.len() as f64;

        g1 = alpha * average_distance;
    }

    // length of path
    if beta == 0.0 {
        g2 = 0.0;
    } else {
        g2 = beta * (path.len() - 1) as f64;
    }

    // semantic distance between last path entity and target entity
    if gamma == 0.0 {
        h = 0.0;
    } else {
        h = gamma
            * store_connector.get_semantic_distance(
                &path.last().unwrap(),
                directional_target_entity,
            );
    }

    costs = g1 + g2 + h;

    // costs cannot be negative
    assert!(costs >= 0.0);

    // workaround required because priority_queue crate only accepts integer costs
    // first an offset is added such that all costs start with a leading 1
    let offset_costs = costs + 1.0;

    // the float is interpreted as its mantissa by removing the dot
    // also, the costs are limited to 10 places such that correct order is retained
    let mut clean_costs_string = offset_costs.to_string().replace(".", "");
    if clean_costs_string.len() < 10 {
        let missing = "0".repeat(10 - clean_costs_string.len());
        clean_costs_string += &missing;
    }
    let integer_const: i64 = clean_costs_string[..10].parse().unwrap();

    assert!(integer_const >= 1000000000);

    return integer_const;
}
