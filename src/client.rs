use std::{error::Error, fmt::Display};

use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;

use crate::{
    collection::CollectionGetBody, molecule::MoleculeGetBody,
    procedure::ProcedureGetBody,
};

#[derive(Deserialize)]
pub struct Information {
    pub query_limit: usize,
}

pub struct FractalClient {
    address: &'static str,
    headers: HeaderMap,
    client: Client,
}

#[derive(Debug)]
struct ClientError;

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClientError")
    }
}

impl Error for ClientError {}

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

    pub async fn get_information(&self) -> Result<Information, Box<dyn Error>> {
        let url = format!("{}information", self.address);
        let response = self
            .client
            .get(url)
            .headers(self.headers.clone())
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(Box::new(ClientError));
        }
        let info: Information = response.json().await?;
        Ok(info)
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

    pub async fn get_molecule(
        &self,
        collection: MoleculeGetBody,
    ) -> reqwest::Response {
        let url = format!("{}molecule", self.address);
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
