use reqwest::{header::HeaderMap, Client};

use crate::collection::CollectionGetBody;

const ADDR: &str = "https://api.qcarchive.molssi.org:443/";

struct FractalClient {
    address: &'static str,
    headers: HeaderMap,
    client: Client,
}

impl FractalClient {
    fn new() -> Self {
        let mut ret = Self {
            address: ADDR,
            headers: HeaderMap::new(),
            client: Client::new(),
        };
        ret.headers
            .insert("Content-Type", "application/json".parse().unwrap());
        ret.headers
            .insert("User-Agent", "qcportal/0.15.7".parse().unwrap());
        ret
    }

    async fn get_collection(
        &self,
        collection: CollectionGetBody,
    ) -> reqwest::Response {
        let url = format!("{}collection", self.address);
        self.client
            .get(url)
            .body(collection.to_json().unwrap())
            .headers(self.headers.clone())
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
        pub name: String,
    }

    /// the important fields in a [CollectionGetResponse]
    #[derive(Debug, Deserialize)]
    pub struct DataSet {
        pub id: String,
        pub collection: String,
        pub name: String,
        pub records: HashMap<String, Record>,
    }

    #[derive(Debug, Deserialize)]
    pub struct CollectionGetResponse {
        pub meta: HashMap<String, Value>,
        pub data: Vec<DataSet>,
    }
}

#[tokio::main]
async fn main() {
    let collection = CollectionGetBody::new(
        "torsiondrivedataset",
        "OpenFF multiplicity correction torsion drive data v1.1",
    );
    let client = FractalClient::new();
    let response = client.get_collection(collection).await;
    dbg!(&response);
    println!("{}", response.text().await.unwrap());
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use crate::collection::CollectionGetResponse;

    #[test]
    fn de_response() {
        let s = read_to_string("testfiles/response.json").unwrap();
        let c: CollectionGetResponse = serde_json::from_str(&s).unwrap();
        dbg!(c);
    }
}
