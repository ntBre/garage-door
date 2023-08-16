use procedure::ProcedureGetBody;
use reqwest::{header::HeaderMap, Client};

use collection::CollectionGetBody;

use crate::{
    collection::CollectionGetResponse, procedure::ProcedureGetResponse,
};

mod collection;
mod procedure;

struct FractalClient {
    address: &'static str,
    headers: HeaderMap,
    client: Client,
}

impl FractalClient {
    fn new() -> Self {
        const ADDR: &str = "https://api.qcarchive.molssi.org:443/";
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

    async fn get_information(&self) {
        let url = format!("{}information", self.address);
        let response = self
            .client
            .get(url)
            .headers(self.headers.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
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

    async fn get_procedure(
        &self,
        collection: ProcedureGetBody,
    ) -> reqwest::Response {
        let url = format!("{}procedure", self.address);
        self.client
            .get(url)
            .body(collection.to_json().unwrap())
            .headers(self.headers.clone())
            .send()
            .await
            .unwrap()
    }
}

#[tokio::main]
async fn main() {
    let col = CollectionGetBody::new(
        "torsiondrivedataset",
        "OpenFF multiplicity correction torsion drive data v1.1",
    );
    let client = FractalClient::new();
    // this doesn't really seem necessary, but they do call it
    client.get_information().await;
    let response: CollectionGetResponse =
        client.get_collection(col).await.json().await.unwrap();

    let proc = ProcedureGetBody::new(response.ids());
    let mut response: ProcedureGetResponse =
        client.get_procedure(proc).await.json().await.unwrap();
    // only keep the complete records
    response.data.retain(|r| r.status.is_complete());

    let _optimization_ids = response.optimization_ids();
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use crate::{
        collection::CollectionGetResponse, procedure::ProcedureGetResponse,
    };

    #[test]
    fn de_response() {
        let s = read_to_string("testfiles/response.json").unwrap();
        let c: CollectionGetResponse = serde_json::from_str(&s).unwrap();
        dbg!(c);
    }

    #[test]
    fn de_procedure() {
        let s = read_to_string("testfiles/procedure.json").unwrap();
        let c: ProcedureGetResponse = serde_json::from_str(&s).unwrap();
        dbg!(c);
    }
}
