extern crate hyper;

use std::str::FromStr;

use hyper::{Client, Uri};
pub struct Thing {
    name: String
}

impl Thing {
    pub fn new(name: &str) -> Thing {
        return Thing {
            name: name.to_string()
        };
    }

    pub fn update(&mut self, msg: &str) -> hyper::client::ResponseFuture {
        let client = Client::new();
        let url = format!("http://dweet.io/dweet/for/{}?msg={}", self.name, msg);
        return client.get(Uri::from_str(&url).unwrap());
    }
}