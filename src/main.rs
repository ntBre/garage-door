#![allow(unused)]

use std::collections::HashMap;

use collection::CollectionGetResponse;
use reqwest::Client;
use serde::Serialize;

use crate::collection::CollectionGetBody;

const ADDR: &str = "https://api.qcarchive.molssi.org:443/";

struct FractalClient {
    address: &'static str,
    headers: HashMap<String, String>,
    encoding: &'static str,
    client: Client,
}

impl FractalClient {
    fn new() -> Self {
        let mut ret = Self {
            address: ADDR,
            headers: HashMap::new(),
            encoding: "msgpack-ext",
            client: Client::new(),
        };
        ret.headers.insert(
            "Content-Type".to_owned(),
            "application/msgpack-ext".to_owned(),
        );
        ret.headers
            .insert("User-Agent".to_owned(), "qcportal/0.15.7".to_owned());
        ret
    }

    async fn get(&self, collection: CollectionGetBody) -> reqwest::Response {
        let url = format!("{}collection", self.address);
        self.client
            .get(url)
            .body(collection.to_json().unwrap())
            .header("Content-Type", "application/json")
            .send()
            .await
            .unwrap()
    }
}

mod collection {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize)]
    struct QueryFilter {
        include: Option<bool>,
        exclude: Option<bool>,
    }

    #[derive(Serialize)]
    struct Data {
        collection: String,
        name: String,
    }

    #[derive(Serialize)]
    pub struct CollectionGetBody {
        meta: QueryFilter,
        data: Data,
    }

    impl CollectionGetBody {
        pub fn new(
            collection: impl Into<String>,
            name: impl Into<String>,
        ) -> Self {
            Self {
                meta: QueryFilter {
                    include: None,
                    exclude: None,
                },
                data: Data {
                    collection: collection.into(),
                    name: name.into(),
                },
            }
        }

        pub fn to_json(&self) -> Result<String, serde_json::Error> {
            serde_json::to_string(&self)
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct Record {
        name: String,
    }

    /// the important fields in a [CollectionGetResponse]
    #[derive(Debug, Deserialize)]
    pub struct DataSet {
        id: String,
        collection: String,
        name: String,
        records: HashMap<String, Record>,
    }

    #[derive(Debug, Deserialize)]
    pub struct CollectionGetResponse {
        meta: HashMap<String, Value>,
        data: Vec<DataSet>,
    }
}

#[tokio::main]
async fn main() {
    let collection = CollectionGetBody::new(
        "torsiondrivedataset",
        "OpenFF multiplicity correction torsion drive data v1.1",
    );
    let client = FractalClient::new();
    let response = client.get(collection).await;
    let response: CollectionGetResponse = response.json().await.unwrap();
    dbg!(response);
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn de_response() {
        let s = read_to_string("response.json").unwrap();
        let c: CollectionGetResponse = serde_json::from_str(&s).unwrap();
        dbg!(c);
    }
}
