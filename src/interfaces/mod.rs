pub mod tcp_interface;

use std::collections::HashMap;
use itertools::Itertools;

pub struct CommIds {
    id_map: HashMap<String, i32>,
    name_map: HashMap<i32, String>,
}

impl CommIds {
    pub fn new() -> Self {
        Self {
            id_map: HashMap::new(),
            name_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: i32, name: &str) {
        self.id_map.insert(name.to_owned(), id);
        self.name_map.insert(id, name.to_string());
    }

    pub fn insert_all(&mut self, comm_ids: &Vec<(String, i32)>) {
        self.name_map.extend(comm_ids.iter().map(|(name, id)| (id.clone(), name.clone())).collect_vec());
        self.id_map.extend(comm_ids.clone());
    }

    pub fn insert_map(&mut self, comm_ids: &HashMap<String, i32>) {
        self.name_map.extend(comm_ids.iter().map(|(name, id)| (id.clone(), name.clone())).collect_vec());
        self.id_map.extend(comm_ids.clone());
    }

    pub fn get_id(&self, name: &str) -> i32 {
        *self.id_map.get(name).expect("Name for unregistered communication id used")
    }

    pub fn get_name(&self, id: i32) -> &str {
        self.name_map.get(&id).expect("Unregistered communication id used")
    }
}