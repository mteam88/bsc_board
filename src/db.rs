// Simple in memory database for storing all events for later display

pub mod db {

    use ethers::types::H256;

    pub struct Db {
        pub events: Vec<Msg>,
    }

    impl Db {
        pub fn new() -> Self {
            Self {
                events: Vec::new(),
            }
        }

        pub fn add_event(&mut self, hash: H256, event: String) {
            self.events.push(Msg { hash, event });
        }

        pub fn get_events_vec(&self) -> Vec<(H256, String)> {
            self.events
                .iter()
                .map(|msg| (msg.hash, msg.event.clone()))
                .collect()
        }
    }
    pub struct Msg {
        hash: H256,
        event: String,
    }
}