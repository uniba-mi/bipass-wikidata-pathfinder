use log::{debug, info};
use priority_queue::DoublePriorityQueue; // allows to extract minimum in contrast to PriorityQueue
use std::collections::{HashMap, HashSet};

use crate::costs_calculator::calculate_costs;
use crate::store_connector::StoreConnector;

#[derive(PartialEq)]
enum Direction {
    FromSourceToTarget,
    FromTargetToSource,
}
pub struct Pathfinder<'a> {
    store_connector: &'a StoreConnector<'a>,
    entity_limit: usize,
}

impl<'a> Pathfinder<'a> {
    pub fn new(store_connector: &'a StoreConnector<'a>, entity_limit: usize) -> Self {
        // create Pathfinder instance with struct fields
        Self {
            store_connector,
            entity_limit,
        }
    }

    pub fn find_path(
        &self,
        source_entity: &str,
        target_entity: &str,
        hyperparameter_config: &(f64, f64, f64),
    ) -> (Vec<String>, Vec<String>, usize) {
        // initialize mappings and adjacency list based on source and target entity
        self.store_connector.get_adjacent_entities(source_entity);
        self.store_connector.get_adjacent_entities(target_entity);

        info!(
            "***** Search path between {} ({}) and {} ({}) using alpha={}, beta={}, gamma={}",
            source_entity,
            self.store_connector.get_label(source_entity),
            target_entity,
            self.store_connector.get_label(target_entity),
            hyperparameter_config.0,
            hyperparameter_config.1,
            hyperparameter_config.2
        );

        // initialize set of visited entities that is used to check if entity limit is reached
        let mut visited_entities: HashSet<String> = HashSet::new();

        // initialize found path
        let mut found_path_forwards: Vec<String> = vec![];
        let mut found_path_backwards: Vec<String> = vec![];
        let mut props_forwards: Vec<String> = vec![];
        let mut props_backwards: Vec<String> = vec![];

        // initialize data structures for direction source -> target
        let mut costs_from_source: HashMap<String, i64> = HashMap::new();
        let mut came_from_source: HashMap<String, String> = HashMap::new();
        let mut prev_prop_from_source: HashMap<String, String> = HashMap::new();
        let mut queue_from_source: DoublePriorityQueue<String, i64> = DoublePriorityQueue::new();

        // push source entity into priority queue
        costs_from_source.insert(source_entity.to_owned(), 0);

        queue_from_source.push(
            source_entity.to_owned(),
            costs_from_source.get(source_entity).unwrap().to_owned(),
        );

        // initialize data structures for direction target -> source
        let mut costs_from_target: HashMap<String, i64> = HashMap::new();
        let mut came_from_target: HashMap<String, String> = HashMap::new();
        let mut prev_prop_from_target: HashMap<String, String> = HashMap::new();
        let mut queue_from_target: DoublePriorityQueue<String, i64> = DoublePriorityQueue::new();

        // push target entity into priority queue
        costs_from_target.insert(target_entity.to_owned(), 0);

        queue_from_target.push(
            target_entity.to_owned(),
            costs_from_target.get(target_entity).unwrap().to_owned(),
        );

        while !(queue_from_source.is_empty() && queue_from_target.is_empty())
            && visited_entities.len() < self.entity_limit
        {
            let current_entity; // the entity currently being visited
            let costs; // the costs of the path leading to the current entity
            let direction; // the direction of the path

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

            // construct path with newly added entity
            let (path, props) = match direction {
                Direction::FromSourceToTarget => self.reconstruct_path(
                    &came_from_source,
                    &prev_prop_from_source,
                    &current_entity,
                ),
                Direction::FromTargetToSource => self.reconstruct_path(
                    &came_from_target,
                    &prev_prop_from_target,
                    &current_entity,
                ),
            };

            let empty_vec: Vec<String> = vec![];

            debug!(
                "*** Processing path {} ({})",
                self.print_path(&path, &empty_vec, &props, &empty_vec),
                if direction == Direction::FromSourceToTarget {
                    "source -> target"
                } else {
                    "target -> source"
                }
            );
            debug!(
                "Costs {} using alpha={}, beta={}, gamma={}",
                costs, hyperparameter_config.0, hyperparameter_config.1, hyperparameter_config.2
            );
            debug!("{} entity/entities visited", visited_entities.len());

            // success detection
            // source -> target: check if the current entity equals the target
            if direction == Direction::FromSourceToTarget && current_entity == target_entity {
                debug!("Direct path from source to target entity found.");
                found_path_forwards = path;
                props_forwards = props;
                break;
            }

            // target -> source: check if the current entity equals the source
            if direction == Direction::FromTargetToSource && current_entity == source_entity {
                debug!("Direct path from target to source entity found.");
                found_path_backwards = path;
                props_backwards = props;
                break;
            }

            // source -> intersecting <- target: check if the current entity is present in both came from mappings
            if came_from_source.contains_key(&current_entity)
                && came_from_target.contains_key(&current_entity)
            {
                debug!(
                    "Path found via an intersection on entity {}.",
                    current_entity
                );

                (found_path_forwards, props_forwards) = self.reconstruct_path(
                    &came_from_source,
                    &prev_prop_from_source,
                    &current_entity,
                );
                (found_path_backwards, props_backwards) = self.reconstruct_path(
                    &came_from_target,
                    &prev_prop_from_target,
                    &current_entity,
                );
                break;
            }

            // set mappings depending on direction
            let came_from = match direction {
                Direction::FromSourceToTarget => &mut came_from_source,
                Direction::FromTargetToSource => &mut came_from_target,
            };

            let prev_prop = match direction {
                Direction::FromSourceToTarget => &mut prev_prop_from_source,
                Direction::FromTargetToSource => &mut prev_prop_from_target,
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
            for (prop, adjacent_entity) in
                self.store_connector.get_adjacent_entities(&current_entity)
            {
                // cycle detection
                if path.contains(&adjacent_entity) {
                    continue;
                }

                // construct new candidate path
                let mut candidate_path = path.clone();
                candidate_path.push(adjacent_entity.clone());
                
                let mut candidate_props = path.clone();
                candidate_props.push(prop.clone());

                // calculate costs of path
                let tentative_costs = calculate_costs(
                    self.store_connector,
                    source_entity,
                    target_entity,
                    &candidate_path,
                    &candidate_props,
                    hyperparameter_config,
                );

                // update mappings with respect to path costs
                if !costs.contains_key(&adjacent_entity)
                    || tentative_costs < costs.get(&adjacent_entity).unwrap().to_owned()
                {
                    came_from.insert(adjacent_entity.clone(), current_entity.to_owned());
                    prev_prop.insert(adjacent_entity.clone(), prop);
                    costs.insert(adjacent_entity.clone(), tentative_costs);

                    // insert adjacent entity in queue; update to lower costs if entity is already present
                    queue.push_decrease(adjacent_entity, tentative_costs);
                }
            }

            debug!(
                "{}/{} entities are in queue_from_source/queue_from_target.",
                queue_from_source.len(),
                queue_from_target.len()
            );
        }

        if found_path_forwards.is_empty() && found_path_backwards.is_empty() {
            info!("No path could be found. :(");
        } else {
            info!(
                "A path was found: {}",
                self.print_path(
                    &found_path_forwards,
                    &found_path_backwards,
                    &props_forwards,
                    &props_backwards
                )
            );
        }

        (
            found_path_forwards,
            found_path_backwards,
            visited_entities.len(),
        )
    }

    fn reconstruct_path<'c>(
        &self,
        came_from: &HashMap<String, String>,
        prev_prop: &HashMap<String, String>,
        current_entity: &'c str,
    ) -> (Vec<String>, Vec<String>) {
        let mut current_entity = current_entity;
        let mut path = vec![current_entity.to_string()];
        let mut props: Vec<String> = vec![];

        while let Some(next) = came_from.get(current_entity) {
            props.push(prev_prop.get(current_entity).unwrap().to_string());
            current_entity = next;
            path.push(current_entity.to_string());
        }

        path.reverse();
        props.reverse();

        debug!(
            "reconstruct_path received current_entity {} and returned {:?} with props {:?}.",
            current_entity, path, props
        );

        // a path features n entities and exactly n-1 properties
        assert!(path.len() == props.len() + 1);

        (path, props)
    }

    // Returns a pretty string representation of the path.
    fn print_path(
        &self,
        path_forwards: &Vec<String>,
        path_backwards: &Vec<String>,
        props_forwards: &Vec<String>,
        props_backwards: &Vec<String>,
    ) -> String {
        let mut path_string;

        if !path_forwards.is_empty() {
            path_string = format!(
                "{} ({})",
                path_forwards.first().unwrap(),
                self.store_connector
                    .get_label(path_forwards.first().unwrap())
            );
        } else {
            path_string = format!(
                "{} ({})",
                path_backwards.last().unwrap(),
                self.store_connector
                    .get_label(path_backwards.last().unwrap())
            );
        }

        for (prop, entity) in props_forwards.iter().zip(path_forwards.iter().skip(1)) {
            path_string += &format!(
                " -{} ({})-> {} ({})",
                prop,
                self.store_connector.get_label(prop),
                entity,
                self.store_connector.get_label(entity)
            )
        }

        for (prop, entity) in props_backwards
            .iter()
            .rev()
            .zip(path_backwards.iter().rev().skip(1))
        {
            path_string += &format!(
                " <-{} ({})- {} ({})",
                prop,
                self.store_connector.get_label(prop),
                entity,
                self.store_connector.get_label(entity)
            )
        }

        path_string
    }
}
