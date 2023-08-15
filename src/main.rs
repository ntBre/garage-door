#![allow(unused)]

use std::collections::HashMap;

use serde::Serialize;

const ADDR: &str = "https://api.qcarchive.molssi.org:443/";

struct FractalClient {
    address: &'static str,
    headers: HashMap<String, String>,
    encoding: &'static str,
}

impl FractalClient {
    fn new() -> Self {
        let mut ret = Self {
            address: ADDR,
            headers: HashMap::new(),
            encoding: "msgpack-ext",
        };
        ret.headers.insert(
            "Content-Type".to_owned(),
            "application/msgpack-ext".to_owned(),
        );
        ret.headers
            .insert("User-Agent".to_owned(), "qcportal/0.15.7".to_owned());
        ret
    }
}

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
struct CollectionGetBody {
    meta: QueryFilter,
    data: Data,
}

#[tokio::main]
async fn main() {
    let collection = CollectionGetBody {
        meta: QueryFilter {
            include: None,
            exclude: None,
        },
        data: Data {
            collection: "torsiondrivedataset".to_owned(),
            name: "OpenFF multiplicity correction torsion drive data v1.1"
                .to_owned(),
        },
    };
    let buf = serde_json::to_string(&collection).unwrap();
    let frac = FractalClient::new();
    let url = format!("{}collection", frac.address);
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .body(buf)
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap();
    dbg!(response);
}
