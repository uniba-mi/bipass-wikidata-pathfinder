use log::{debug, info};
use priority_queue::DoublePriorityQueue;
use std::collections::{HashMap, HashSet};

use crate::store_connector::StoreConnector;

#[derive(PartialEq)]
enum Direction {
    FromSourceToTarget,
    FromTargetToSource,
}
pub struct Pathfinder<'a> {
    store_connector: &'a StoreConnector<'a>,
    distance_method: String,
    entity_limit: usize,
}

impl<'a> Pathfinder<'a> {
    pub fn new(
        store_connector: &'a StoreConnector<'a>,
        distance_method: String,
        entity_limit: usize,
    ) -> Self {
        // create Pathfinder instance with struct fields
        Self {
            store_connector,
            distance_method,
            entity_limit,
        }
    }

    pub fn find_path(
        &self,
        source_entity: &str,
        target_entity: &str,
        search_params: (f64, f64, f64),
    ) -> (Vec<String>, Option<Vec<String>>, f64, usize) {
        // initialize mappings and adjacency list based on source and target entity
        self.store_connector.get_adjacent_entities(source_entity);
        self.store_connector.get_adjacent_entities(target_entity);

        info!(
            "***** Search path between {} ({}) and {} ({}) using alpha={}, beta={}, gamma={}",
            source_entity,
            self.store_connector.get_label(source_entity),
            target_entity,
            self.store_connector.get_label(target_entity),
            search_params.0,
            search_params.1,
            search_params.2
        );

        // initialize set of visited entities that is used to check if entity limit is reached
        let mut visited_entities: HashSet<String> = HashSet::new();

        // initialize found path and intersecting entity
        let mut found_path_forwards: Vec<String> = vec![];
        let mut found_path_backwards: Option<Vec<String>> = None;

        // initialize data structures for direction source -> target
        let mut costs_from_source: HashMap<String, i64> = HashMap::new();
        let mut came_from_source: HashMap<String, String> = HashMap::new();
        let mut queue_from_source: DoublePriorityQueue<String, i64> = DoublePriorityQueue::new();

        // push source entity into priority queue
        costs_from_source.insert(
            source_entity.to_owned(),
            self.calculate_costs(
                source_entity,
                target_entity,
                self.reconstruct_path(came_from_source.clone(), source_entity),
                search_params,
            ),
        );

        queue_from_source.push(
            source_entity.to_owned(),
            costs_from_source.get(source_entity).unwrap().to_owned(),
        );

        // initialize data structures for direction target -> source
        let mut costs_from_target: HashMap<String, i64> = HashMap::new();
        let mut came_from_target: HashMap<String, String> = HashMap::new();
        let mut queue_from_target: DoublePriorityQueue<String, i64> = DoublePriorityQueue::new();

        // push target entity into priority queue
        costs_from_target.insert(
            target_entity.to_owned(),
            self.calculate_costs(
                source_entity,
                target_entity,
                self.reconstruct_path(came_from_target.clone(), target_entity),
                search_params,
            ),
        );

        queue_from_target.push(
            target_entity.to_owned(),
            costs_from_target.get(target_entity).unwrap().to_owned(),
        );

        while !(queue_from_source.is_empty() && queue_from_target.is_empty())
            && visited_entities.len() < self.entity_limit
        {
            let current_entity;
            let costs;

            // states wether currently observed path is from source to target (st) or from target to source (ts)
            // let direction;
            let direction;

            // find the entity with the least costly associated path in both queues
            // case 1: both queues are not empty
            if !(queue_from_source.is_empty() || queue_from_target.is_empty()) {
                if queue_from_source.peek_min().unwrap().1
                    <= queue_from_target.peek_min().unwrap().1
                {
                    (current_entity, costs) = queue_from_source.pop_min().unwrap();
                    direction = Direction::FromSourceToTarget;
                } else {
                    (current_entity, costs) = queue_from_target.pop_min().unwrap();
                    direction = Direction::FromTargetToSource;
                }
            // case 2: queue from source is empty
            } else if queue_from_source.is_empty() {
                (current_entity, costs) = queue_from_target.pop_min().unwrap();
                direction = Direction::FromTargetToSource;
            // case 3: queue from target is empty
            } else {
                (current_entity, costs) = queue_from_source.pop_min().unwrap();
                direction = Direction::FromSourceToTarget;
            }

            visited_entities.insert(current_entity.clone());

            // reconstruct path
            let mut path = match direction {
                Direction::FromSourceToTarget => {
                    self.reconstruct_path(came_from_source.clone(), &current_entity)
                }
                Direction::FromTargetToSource => {
                    self.reconstruct_path(came_from_target.clone(), &current_entity)
                }
            };

            debug!(
                "*** Processing path {} ({})",
                self.path_to_string(&path, &None),
                if direction == Direction::FromSourceToTarget {
                    "source -> target"
                } else {
                    "target -> source"
                }
            );
            debug!(
                "Costs {} using alpha={}, beta={}, gamma={}",
                costs, search_params.0, search_params.1, search_params.2
            );
            debug!("{} entity/entities visited", visited_entities.len());

            // path detection
            // source -> target: check if the current entity equals the target
            if direction == Direction::FromSourceToTarget && current_entity == target_entity {
                debug!("Direct path from source to target entity found.");
                found_path_forwards = path;
                break;
            }

            // target -> source: check if the current entity equals the source
            if direction == Direction::FromTargetToSource && current_entity == source_entity {
                debug!("Direct path from target to source entity found.");
                path.reverse();
                found_path_forwards = path;
                break;
            }

            // target -> intersecting <- source: check if the current entity is present in both came from mappings
            if came_from_source.contains_key(&current_entity)
                && came_from_target.contains_key(&current_entity)
            {
                debug!(
                    "Path found via an intersection on entity {}.",
                    current_entity
                );
                found_path_forwards = self.reconstruct_path(came_from_source, &current_entity);

                let found_path_fragment = self.reconstruct_path(came_from_target, &current_entity);

                found_path_backwards = Some(found_path_fragment);
                break;
            }

            // set mappings correctly depending on direction
            let came_from = match direction {
                Direction::FromSourceToTarget => &mut came_from_source,
                Direction::FromTargetToSource => &mut came_from_target,
            };

            let costs = match direction {
                Direction::FromSourceToTarget => &mut costs_from_source,
                Direction::FromTargetToSource => &mut costs_from_target,
            };

            let queue = match direction {
                Direction::FromSourceToTarget => &mut queue_from_source,
                Direction::FromTargetToSource => &mut queue_from_target,
            };

            // insert adjacent entities into priority queue if they not have been visited before
            for adjacent_entity in self.store_connector.get_adjacent_entities(&current_entity) {
                // cycle detection
                if path.contains(&adjacent_entity.to_owned()) {
                    continue;
                }

                let mut candidate_path = path.clone();
                candidate_path.push(adjacent_entity.clone());

                let tentative_costs = self.calculate_costs(
                    source_entity,
                    target_entity,
                    candidate_path,
                    search_params,
                );

                if !costs.contains_key(&adjacent_entity)
                    || tentative_costs < costs.get(&adjacent_entity).unwrap().to_owned()
                {
                    came_from.insert(adjacent_entity.clone(), current_entity.to_owned());
                    costs.insert(adjacent_entity.clone(), tentative_costs);

                    if queue.get(&adjacent_entity) == None {
                        queue.push(adjacent_entity, tentative_costs);
                    }
                }
            }

            debug!(
                "{}/{} entities are in queue_from_source/queue_from_target.",
                queue_from_source.len(),
                queue_from_target.len()
            );
        }

        // calculate score of pathfinder run (lower is better) as the harmonic mean of visited entities and average_semantic distance is used
        let mut score = visited_entities.len() as f64;

        if found_path_forwards.is_empty() {
            info!("No path could be found. :(");
            // double the score to penalize no found path
            score *= 2.0;
        } else {
            info!(
                "A path was found: {}",
                self.path_to_string(&found_path_forwards, &found_path_backwards)
            );
            debug!("Pathfinding score: {}", score);
        }

        (
            found_path_forwards.clone(),
            found_path_backwards.clone(),
            score,
            visited_entities.len()
        )
    }

    fn reconstruct_path<'c>(
        &self,
        came_from: HashMap<String, String>,
        current_entity: &'c str,
    ) -> Vec<String> {
        let mut current_entity = current_entity;
        let mut path = vec![current_entity.to_string()];

        while let Some(next) = came_from.get(current_entity) {
            current_entity = next;
            path.push(current_entity.to_string());
        }

        path.reverse();

        debug!(
            "reconstruct_path received current_entity {} and returned {}.",
            current_entity,
            path.join(", ")
        );

        path
    }

    // TODO Improve costs function; maybe take property quality into account?
    // Calculates the costs of a path comprising entities.
    // Costs mapping fScore from https://en.wikipedia.org/wiki/A*_search_algorithm cannot be used for us as we use the average (!) semantic distance in the g costs
    fn calculate_costs(
        &self,
        source_entity: &str,
        target_entity: &str,
        path: Vec<String>,
        search_params: (f64, f64, f64),
    ) -> i64 {
        let (alpha, beta, gamma) = search_params;
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
            g1 = alpha * self.calculate_average_distance(path_so_far, directional_target_entity)
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
                * self.store_connector.get_semantic_distance(
                    &path.last().unwrap(),
                    directional_target_entity,
                    &self.distance_method,
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

    // Calculates the average semantic distance between each entity in the provided path and the provided entity.
    fn calculate_average_distance(&self, path: &[String], entity: &str) -> f64 {
        let total_distance: f64 = path.iter().fold(0.0, |acc, e| {
            acc + self
                .store_connector
                .get_semantic_distance(e, entity, &self.distance_method)
        });

        total_distance / path.len() as f64
    }

    // Returns a pretty string representation of the forwards path.
    // In case of a path with an intersection use the optional backwards path parameter.
    fn path_to_string(
        &self,
        forwards_path: &Vec<String>,
        backwards_path: &Option<Vec<String>>,
    ) -> String {
        let mut fragment: Vec<String> = forwards_path
            .iter()
            .map(|entity| format!("{} ({})", entity, self.store_connector.get_label(entity)))
            .collect();

        let mut path_string = fragment.join(" -> ");

        if backwards_path.is_some() {
            let mut the_backwards_path = backwards_path.as_ref().unwrap().clone();
            the_backwards_path.pop();
            the_backwards_path.reverse();

            fragment = the_backwards_path
                .iter()
                .map(|entity| {
                    format!(
                        " <- {} ({})",
                        entity,
                        self.store_connector.get_label(entity)
                    )
                })
                .collect();

            path_string += &fragment.join("");
        }

        path_string
    }
}
