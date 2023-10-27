use log::debug;
use serde_json::{Map, Value};
use std::str::FromStr;

use cached::proc_macro::cached;

/// A connector for interacting with APIs
pub struct ApiConnector {
    /// The base URL of the word embedding API
    wembed_api: String,
    /// The base URL of the Wikidata API
    wikidata_api: String,
}

impl ApiConnector {
    /// Creates an ApiConnector instance with the required fields.
    /// # Arguments
    /// * `wembed_api` - The base URL of the word embedding API
    /// * `wikidata_api` - The base URL of the Wikidata API
    /// # Returns
    /// * The instance
    pub fn new(wembed_api: String, wikidata_api: String) -> Self {
        Self {
            wembed_api,
            wikidata_api,
        }
    }

    /// Fetches the ids, labels, and descriptions of entities adjacent to an entity from our Wikidata API.
    /// The depth parameter reduces the load on the Wikidata SPARQL endpoint by pre-fetching more entities and labels.
    /// # Arguments
    /// * `entity` - The entity
    /// # Returns
    /// * A mapping between entity IDs and labels
    /// * A mapping between entity IDs and descriptions
    /// * A mapping between property IDs and labels
    /// * A mapping between entity IDs and lists with IDs of adjacent entities
    pub fn fetch_adjacent_entity_data(
        &self,
        entity: &str,
    ) -> (
        Map<String, Value>,
        Map<String, Value>,
        Map<String, Value>,
        Map<String, Value>,
        Map<String, Value>,
    ) {
        let mut json = Default::default();

        // closure for making the request using a depth parameter
        let mut make_request = |d: i8| -> Result<(), reqwest::Error> {
            let url = format!(
                "{}/adjacent_entities?entity={}&depth={}",
                self.wikidata_api, entity, d
            );
            let response = reqwest::blocking::get(url)?.text()?;
            json = Value::from_str(&response).unwrap();
            Ok(())
        };

        // try with higher depth first and decrement if it fails
        for depth in (1..=2).rev() {
            debug!(
                "Attempting to fetch adjacent entities of {} with depth {}",
                entity, depth
            );
            let result = make_request(depth);

            if result.is_ok() {
                result.unwrap();
                break
            }
        }

        // extract the data for all adjacent entities
        let adjacent_entities_data = json.get("adjacent_entities").unwrap().as_object().unwrap();
        let q_label_data = json.get("q_labels").unwrap().as_object().unwrap();
        let q_desc_data = json.get("q_descriptions").unwrap().as_object().unwrap();
        let p_label_data = json.get("p_labels").unwrap().as_object().unwrap();
        let p_desc_data = json.get("p_descriptions").unwrap().as_object().unwrap();

        return (
            q_label_data.clone(),
            q_desc_data.clone(),
            p_label_data.clone(),
            p_desc_data.clone(),
            adjacent_entities_data.clone(),
        );
    }

    /// Fetches the label and the description of an entity.
    /// # Arguments
    /// * `entity` - The entity
    /// # Returns
    /// * The label of the entity
    /// * The description of the entity
    pub fn fetch_label_description(&self, entity: &str) -> (String, String) {
        let url = format!("{}/label_description?entity={}", self.wikidata_api, entity);
        let response = reqwest::blocking::get(url).unwrap().text().unwrap();

        let mut fetched_label = "";
        let mut fetched_description = "";

        let parse_result = Value::from_str(&response);
        if parse_result.is_ok() {
            let json = parse_result.unwrap();
            fetched_label = json.get("label").unwrap().as_str().unwrap();
            fetched_description = json.get("description").unwrap().as_str().unwrap();

            return (fetched_label.to_owned(), fetched_description.to_owned())
        }

        (fetched_label.to_owned(), fetched_description.to_owned())
    }

    /// Fetches the semantic distance between two strings.
    /// To optimize the runtime, a cached function is called.
    /// # Arguments
    /// * `string_a` - The first string
    /// * `string_b` - The second string
    /// # Returns
    /// * The semantic distance
    pub fn fetch_semantic_distance(&self, string_a: &str, string_b: &str) -> f64 {
        fetch_distance_cached(
            (&self.wembed_api).to_owned(),
            string_a.to_owned(),
            string_b.to_owned(),
        )
    }
}

/// A cached function retrieving the semantic distance between two strings via the word embedding API.
/// The rationale for introducing the separate function is that functions using the cached procedural macro do not allow &self as a parameter.
/// # Arguments
/// * `wembed_api` - The base URL of the word embedding API
/// * `string_a` - The first string
/// * `string_b` - The second string
/// # Returns
/// * The semantic distance
#[cached]
pub fn fetch_distance_cached(wembed_api: String, string_a: String, string_b: String) -> f64 {
    let url = format!("{}?string_a={}&string_b={}", wembed_api, string_a, string_b);

    let response = reqwest::blocking::get(url).unwrap().text().unwrap();
    let json = Value::from_str(&response).unwrap();
    let fetched_distance = json.get("distance").unwrap().as_f64().unwrap();

    fetched_distance
}
