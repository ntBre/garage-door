use std::{collections::HashMap, error::Error, fmt::Display};

use futures::{future::join_all, Future};
use reqwest::{header::HeaderMap, Client, Response};
use serde::{Deserialize, Serialize};

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

pub trait Body {
    fn new(ids: Vec<String>) -> Self;
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

    pub async fn get_collection(
        &self,
        body: CollectionGetBody,
    ) -> CollectionGetResponse {
        self.get("collection", body).await.json().await.unwrap()
    }

    pub async fn get_procedure<T: for<'a> Deserialize<'a>>(
        &self,
        body: ProcedureGetBody,
    ) -> ProcedureGetResponse<T> {
        self.get("procedure", body).await.json().await.unwrap()
    }

    pub async fn get_molecule(
        &self,
        body: MoleculeGetBody,
    ) -> MoleculeGetResponse {
        self.get("molecule", body).await.json().await.unwrap()
    }

    async fn get_query_limit(&self) -> usize {
        self.get_information().await.unwrap().query_limit
    }

    async fn get_chunked<'a, B, R, F, Q>(
        &'a self,
        method: Q,
        ids: &[String],
        chunk_size: usize,
    ) -> Vec<R>
    where
        B: Body,
        F: Future<Output = R>,
        Q: Fn(&'a FractalClient, B) -> F,
    {
        let mut futures = Vec::new();
        for chunk in ids.chunks(chunk_size) {
            let proc = B::new(chunk.to_vec());
            futures.push(method(self, proc));
        }
        join_all(futures).await
    }

    pub async fn retrieve_dataset(
        &self,
        collection_request: CollectionGetBody,
    ) -> Vec<(String, String, Vec<Vec<f64>>)> {
        let start = std::time::Instant::now();

        // get the query_limit and the initial collection
        let (query_limit, collection) = tokio::join! {
            self.get_query_limit(),
            self.get_collection(collection_request),
        };

        let records: Vec<TorsionDriveRecord> = self
            .get_chunked(Self::get_procedure, &collection.ids(), query_limit)
            .await
            .into_iter()
            .flat_map(|r: ProcedureGetResponse<TorsionDriveRecord>| r.data)
            .filter(|r| r.status.is_complete())
            .collect();

        eprintln!("{} torsion drive records", records.len());

        // this is a map of optimization_id -> (record_id, grid_id)
        let mut intermediate_ids = HashMap::new();
        for record in &records {
            for (grid_id, m) in &record.minimum_positions {
                intermediate_ids.insert(
                    record.optimization_history[grid_id][*m].clone(),
                    (record.id.clone(), grid_id.clone()),
                );
            }
        }
        let optimization_ids: Vec<String> =
            intermediate_ids.keys().cloned().collect();

        // get the optimization records corresponding to each position in the
        // TorsionDrive
        let responses: Vec<ProcedureGetResponse<OptimizationRecord>> = self
            .get_chunked(Self::get_procedure, &optimization_ids, query_limit)
            .await;

        // this is a map of (record_id, grid_id) -> opt_record_id
        let mut molecule_ids = HashMap::new();
        let mut ids = Vec::with_capacity(optimization_ids.len());
        for response in responses {
            for opt_record in &response.data {
                molecule_ids.insert(
                    intermediate_ids[&opt_record.id].clone(),
                    opt_record.final_molecule.clone(),
                );
            }
            ids.extend(response.into_final_molecules());
        }

        eprintln!("asking for {} molecules", ids.len());

        // get the final molecules from each optimization trajectory
        let responses: Vec<MoleculeGetResponse> = self
            .get_chunked(Self::get_molecule, &ids, query_limit)
            .await;

        let mut molecules = HashMap::with_capacity(ids.len());
        for response in responses {
            for molecule in response.data {
                molecules.insert(molecule.id.clone(), molecule);
            }
        }

        eprintln!("received {} molecules", molecules.len());

        eprintln!(
            "execution time: {:.1} s",
            start.elapsed().as_millis() as f64 / 1000.0
        );

        let results: Vec<_> = collection
            .data
            .into_iter()
            .flat_map(|ds| ds.records.into_values())
            .collect();

        make_results(results, records, molecule_ids, molecules)
    }
}
