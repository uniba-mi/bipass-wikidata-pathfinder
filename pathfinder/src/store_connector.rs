use bincode;
use serde::{Deserialize, Serialize};
use sled::{Batch, Db};

use crate::api_connector::ApiConnector;
use log::{debug, warn};

#[derive(Serialize, Deserialize, Debug)]
struct StoreValue {
    data: Vec<String>,
}

pub struct StoreConnector<'a> {
    api_connector: &'a ApiConnector,
    label_mapping: Db,
    desc_mapping: Db,
    distance_mapping: Db,
    adjacency_list: Db,
}

impl<'a> StoreConnector<'a> {
    pub fn new(
        api_connector: &'a ApiConnector,
        label_mapping_path: String,
        desc_mapping_path: String,
        distance_mapping_path: String,
        adjacency_list_path: String,
    ) -> Self {
        // load key value stores
        let db_paths = [
            label_mapping_path,
            desc_mapping_path,
            distance_mapping_path,
            adjacency_list_path,
        ];

        let [label_mapping, desc_mapping, distance_mapping, adjacency_list] =
            db_paths.map(|path| sled::open(path).unwrap());

        // create instance with loaded stores
        Self {
            api_connector,
            desc_mapping,
            label_mapping,
            distance_mapping,
            adjacency_list,
        }
    }

    // Fetches the entities adjacent to the specified entity.
    // If the entity has not been seen before, its data and the data of the adjacent entities is fetched.
    pub fn get_adjacent_entities(&self, entity: &str) -> Vec<(String, String)> {
        if !self.label_mapping.contains_key(entity).unwrap()
            || !self.desc_mapping.contains_key(entity).unwrap()
            || !self.adjacency_list.contains_key(entity).unwrap()
        {
            let (q_label_data, q_desc_data, p_label_data, p_desc_data, adjacent_entities_data) =
                self.api_connector.fetch_adjacent_entity_data(entity);

            let mut batch = Batch::default();

            // update the adjacency list for all retrieved entities
            for (some_entity, its_adjacent_entities) in adjacent_entities_data {
                if !self.adjacency_list.contains_key(&some_entity).unwrap() {
                    let its_adjacent_entities_parsed = its_adjacent_entities.as_array().unwrap();
                    let cleaned_entities: Vec<String> = its_adjacent_entities_parsed
                        .iter()
                        .map(|elem| elem.as_str().unwrap().to_owned())
                        .collect();

                    // a key is a single entity and the value is a vector with elements of this form: some_property-adjacent_entity
                    batch.insert(
                        some_entity.as_str(),
                        bincode::serialize(&cleaned_entities).unwrap(),
                    );
                }
            }

            self.adjacency_list.apply_batch(batch).unwrap();

            // update the label mapping for all retrieved entities and properties
            batch = Batch::default();
            q_label_data.iter().for_each(|(e, l)| {
                batch.insert(e.as_str(), l.as_str().unwrap());
            });

            p_label_data.iter().for_each(|(e, l)| {
                batch.insert(e.as_str(), l.as_str().unwrap());
            });

            self.label_mapping.apply_batch(batch).unwrap();

            // update the desc mapping for all retrieved entities
            batch = Batch::default();
            q_desc_data.iter().for_each(|(e, l)| {
                batch.insert(e.as_str(), l.as_str().unwrap());
            });

            p_desc_data.iter().for_each(|(e, l)| {
                batch.insert(e.as_str(), l.as_str().unwrap());
            });

            self.desc_mapping.apply_batch(batch).unwrap();
        }

        // read from store
        let bytes = self.adjacency_list.get(entity).unwrap().unwrap();
        let raw: Vec<String> = bincode::deserialize(&bytes).unwrap();

        let mut adjacent_entities: Vec<(String, String)> = vec![];

        for entry in raw {
            let split: Vec<&str> = entry.split("-").collect();
            adjacent_entities.push((split[0].to_string(), split[1].to_string()))
        }

        debug!(
            "get_adjacent_entities received entity {} and returned {} entities.",
            entity,
            adjacent_entities.len()
        );

        adjacent_entities
    }

    pub fn get_description(&self, entity: &str) -> String {
        let contains = self.desc_mapping.contains_key(&entity).unwrap();

        let description = match contains {
            true => {
                let value = self.desc_mapping.get(&entity).unwrap().unwrap();
                String::from(std::str::from_utf8(&value).unwrap())
            }
            false => self.fallback_get_label_description(&entity).1,
        };

        description
    }

    pub fn get_label(&self, entity: &str) -> String {
        let contains = self.label_mapping.contains_key(&entity).unwrap();

        let label = match contains {
            true => {
                let value = self.label_mapping.get(&entity).unwrap().unwrap();
                String::from(std::str::from_utf8(&value).unwrap())
            }
            false => self.fallback_get_label_description(&entity).0,
        };

        label
    }

    // semantic distance is calculated using both entity labels and descriptions
    // the descriptions mitigate ambiguity in entity labels
    pub fn get_semantic_distance(&self, entity_a: &str, entity_b: &str) -> f64 {
        let string_a = format!(
            "{} {}",
            self.get_label(entity_a),
            self.get_description(entity_a)
        );
        let string_b = format!(
            "{} {}",
            self.get_label(entity_b),
            self.get_description(entity_b)
        );

        let combined_string = format!("{}&{}", string_a, string_b);

        let contains = self
            .distance_mapping
            .contains_key(&combined_string)
            .unwrap();

        let distance = match contains {
            true => {
                let value = self
                    .distance_mapping
                    .get(&combined_string)
                    .unwrap()
                    .unwrap();
                let value_as_string = String::from(std::str::from_utf8(&value).unwrap());

                value_as_string.parse().unwrap()
            }
            false => {
                let value = self
                    .api_connector
                    .fetch_semantic_distance(&string_a, &string_b);
                let value_as_string = &value.to_string();

                self.distance_mapping
                    .insert(combined_string, value_as_string.as_str())
                    .unwrap();

                value_as_string.parse().unwrap()
            }
        };

        debug!(
            "Semantic distance between {} and {}: {}",
            entity_a, entity_b, distance
        );

        distance
    }

    // Retrieves the average frequency of one or more properties
    // This value is not stored locally
    pub fn get_average_prop_frequency(&self, props: &Vec<String>) -> f64 {
        return self.api_connector.fetch_average_prop_frequency(props);
    }

    // For making fallback request if label or description was not be retrieved before.
    // Label and description of an entity should already have been retrieved when fetching the adjacent entities of the entity pointing to this entity.
    fn fallback_get_label_description(&self, entity: &str) -> (String, String) {
        warn!("Fallback request for label of {} triggered.", entity);
        let (fetched_label, fetched_description) =
            self.api_connector.fetch_label_description(&entity);

        self.label_mapping
            .insert(entity, fetched_label.as_str())
            .unwrap();

        self.desc_mapping
            .insert(entity, fetched_description.as_str())
            .unwrap();

        (fetched_label, fetched_description)
    }
}
