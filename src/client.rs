use reqwest::{header::HeaderMap, Client};

use crate::{collection::CollectionGetBody, procedure::ProcedureGetBody};

pub struct FractalClient {
    address: &'static str,
    headers: HeaderMap,
    client: Client,
}

impl FractalClient {
    pub fn new() -> Self {
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

    pub async fn get_information(&self) {
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

    pub async fn get_collection(
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

    pub async fn get_procedure(
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

impl Default for FractalClient {
    fn default() -> Self {
        Self::new()
    }
}
