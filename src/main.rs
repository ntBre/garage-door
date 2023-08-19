//! the goal here is to replicate the sequence of Python code:
//!
//! ```python
//! client = FractalClient()
//! collection = TorsionDriveResultCollection.from_server(
//!     client=client,
//!     datasets=[
//!         "OpenFF multiplicity correction torsion drive data v1.1",
//!     ],
//!     spec_name="default",
//! )
//! records_and_molecules = collection.to_records()
//! ```
//!
//! Until rodeo can generate Molecules like rdkit, we can't actually replicate
//! the very last line. The best we can do is return the building blocks of
//! Molecules and their conformers, as the docs for [make_results] describe.

use garage_door::make_results;
use std::collections::HashMap;
use tokio::task::JoinSet;

use garage_door::{
    client::FractalClient,
    collection::{CollectionGetBody, CollectionGetResponse},
    molecule::{MoleculeGetBody, MoleculeGetResponse},
    procedure::{
        OptimizationRecord, ProcedureGetBody, ProcedureGetResponse,
        TorsionDriveRecord,
    },
};

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();
    let col = CollectionGetBody::new(
        "torsiondrivedataset",
        "OpenFF multiplicity correction torsion drive data v1.1",
    );
    let client = FractalClient::new();
    let info = client.get_information().await.unwrap();
    eprintln!("query_limit = {}", info.query_limit);
    let response: CollectionGetResponse =
        client.get_collection(col).await.json().await.unwrap();

    let proc = ProcedureGetBody::new(response.ids());

    let results: Vec<_> = response
        .data
        .into_iter()
        .flat_map(|ds| ds.records.into_values())
        .collect();

    let mut records: ProcedureGetResponse<TorsionDriveRecord> =
        client.get_procedure(proc).await.json().await.unwrap();
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
        let c = client.clone();
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
        ids.extend(response.final_molecules());
    }

    // now you have ANOTHER level of indirection: take the final_molecule ids
    // from this last get_procedure call and query for them

    eprintln!("asking for {} molecules", ids.len());

    let mut set = JoinSet::new();
    for chunk in ids.chunks(info.query_limit) {
        let proc = MoleculeGetBody::new(chunk.to_vec());
        let c = client.clone();
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

    make_results(results, records, molecule_ids, molecules);
}
