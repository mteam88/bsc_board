// Simple in memory database for storing all events for later display

pub mod db {
    use std::collections::HashMap;

    use ethers::types::H256;

    pub struct Db {
        pub events: HashMap<H256, String>,
    }

    impl Db {
        pub fn new() -> Self {
            Self {
                events: HashMap::new(),
            }
        }

        pub fn add_event(&mut self, hash: H256, event: String) {
            self.events.insert(hash, event);
        }

        pub fn get_events_vec(&self) -> Vec<(H256, String)> {
            self.events.iter().map(|(k, v)| (*k, v.clone())).collect()
        }
    }
}