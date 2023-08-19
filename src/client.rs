use std::{collections::HashMap, error::Error, fmt::Display};

use reqwest::{header::HeaderMap, Client, Response};
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::{
    collection::{CollectionGetBody, CollectionGetResponse},
    make_results,
    molecule::{MoleculeGetBody, MoleculeGetResponse},
    procedure::{
        OptimizationRecord, ProcedureGetBody, ProcedureGetResponse,
        TorsionDriveRecord,
    },
};

#[derive(Debug)]
struct ClientError;

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClientError")
    }
}

impl Error for ClientError {}

pub trait ToJson {
    fn to_json(&self) -> Result<String, serde_json::Error>;
}

impl<T> ToJson for T
where
    T: Serialize,
{
    fn to_json(&self) -> Result<std::string::String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

#[derive(Deserialize)]
pub struct Information {
    pub query_limit: usize,
}

impl Default for FractalClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
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

    async fn get(&self, endpoint: &str, body: impl ToJson) -> Response {
        let url = format!("{}{endpoint}", self.address);
        let ret = self
            .client
            .get(url)
            .body(body.to_json().unwrap())
            .headers(self.headers.clone())
            .send()
            .await
            .unwrap();
        if !ret.status().is_success() {
            panic!("get `{endpoint}` failed with {ret:?}");
        }
        ret
    }

    pub async fn get_collection(&self, body: CollectionGetBody) -> Response {
        self.get("collection", body).await
    }

    pub async fn get_procedure(&self, body: ProcedureGetBody) -> Response {
        self.get("procedure", body).await
    }

    pub async fn get_molecule(&self, body: MoleculeGetBody) -> Response {
        self.get("molecule", body).await
    }

    pub async fn retrieve_dataset(
        &self,
        col: CollectionGetBody,
    ) -> Vec<(String, String, Vec<Vec<f64>>)> {
        let start = std::time::Instant::now();
        let info = self.get_information().await.unwrap();
        eprintln!("query_limit = {}", info.query_limit);
        let response: CollectionGetResponse =
            self.get_collection(col).await.json().await.unwrap();

        let proc = ProcedureGetBody::new(response.ids());

        let results: Vec<_> = response
            .data
            .into_iter()
            .flat_map(|ds| ds.records.into_values())
            .collect();

        let mut records: ProcedureGetResponse<TorsionDriveRecord> =
            self.get_procedure(proc).await.json().await.unwrap();
        // only keep the complete records
        records.data.retain(|r| r.status.is_complete());

        eprintln!("{} torsion drive records", records.data.len());

        let optimization_ids = records.optimization_ids();

        // this is a map of optimization_id -> (record_id, grid_id)
        let mut intermediate_ids = HashMap::new();
        for record in &records.data {
            for (grid_id, m) in &record.minimum_positions {
                intermediate_ids.insert(
                    record.optimization_history[grid_id][*m].clone(),
                    (record.id.clone(), grid_id.clone()),
                );
            }
        }

        let mut set = JoinSet::new();
        for chunk in optimization_ids.chunks(info.query_limit) {
            let proc = ProcedureGetBody::new(chunk.to_vec());
            let c = self.clone();
            set.spawn(async move { c.get_procedure(proc).await });
        }

        // this is a map of (record_id, grid_id) -> opt_record_id
        let mut molecule_ids = HashMap::new();
        let mut ids = Vec::with_capacity(optimization_ids.len());
        while let Some(response) = set.join_next().await {
            let response: ProcedureGetResponse<OptimizationRecord> =
                response.unwrap().json().await.unwrap();
            for opt_record in &response.data {
                molecule_ids.insert(
                    intermediate_ids[&opt_record.id].clone(),
                    opt_record.final_molecule.clone(),
                );
            }
            ids.extend(response.into_final_molecules());
        }

        // now you have ANOTHER level of indirection: take the final_molecule
        // ids from this last get_procedure call and query for them

        eprintln!("asking for {} molecules", ids.len());

        let mut set = JoinSet::new();
        for chunk in ids.chunks(info.query_limit) {
            let proc = MoleculeGetBody::new(chunk.to_vec());
            let c = self.clone();
            set.spawn(async move { c.get_molecule(proc).await });
        }

        let mut molecules = HashMap::with_capacity(ids.len());
        while let Some(response) = set.join_next().await {
            let response: MoleculeGetResponse =
                response.unwrap().json().await.unwrap();
            for molecule in response.data {
                molecules.insert(molecule.id.clone(), molecule);
            }
        }

        eprintln!("received {} molecules", molecules.len());

        eprintln!(
            "execution time: {:.1} s",
            start.elapsed().as_millis() as f64 / 1000.0
        );

        make_results(results, records, molecule_ids, molecules)
    }
}
