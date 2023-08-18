use std::{collections::HashMap, fs::read_to_string};

use crate::{
    client::FractalClient,
    collection::CollectionGetResponse,
    make_results,
    molecule::{MoleculeGetBody, MoleculeGetResponse},
    procedure::{
        OptimizationRecord, ProcedureGetBody, ProcedureGetResponse,
        TorsionDriveRecord,
    },
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
    let c: ProcedureGetResponse<TorsionDriveRecord> =
        serde_json::from_str(&s).unwrap();
    dbg!(c);
}

#[test]
fn de_opt_procedure() {
    let s = read_to_string("testfiles/opt_procedure.json").unwrap();
    let c: ProcedureGetResponse<OptimizationRecord> =
        serde_json::from_str(&s).unwrap();
    dbg!(c);
}

#[tokio::test]
async fn full() {
    let want = {
        let s = read_to_string("testfiles/final.dat").unwrap();
        let lines = s.lines();
        let mut ret = Vec::new();
        for line in lines {
            let sp: Vec<_> = line.split("=>").map(|s| s.trim()).collect();
            ret.push((
                sp[0].to_owned(),
                sp[1].to_owned(),
                sp[2].parse::<usize>().unwrap(),
            ));
        }
        ret.sort_by_key(|r| r.0.clone());
        ret
    };
    let s = read_to_string("testfiles/response.json").unwrap();
    let c: CollectionGetResponse = serde_json::from_str(&s).unwrap();
    let results: Vec<_> = c
        .data
        .into_iter()
        .flat_map(|ds| ds.records.into_values())
        .collect();
    let s = read_to_string("testfiles/procedure.json").unwrap();
    let mut records: ProcedureGetResponse<TorsionDriveRecord> =
        serde_json::from_str(&s).unwrap();
    records.data.retain(|r| r.status.is_complete());

    // TODO dump this stuff to json/DRY it out. this is all copy pasta from main

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

    let client = FractalClient::new();

    // this is a map of (record_id, grid_id) -> opt_record_id
    let mut molecule_ids = HashMap::new();

    let mut ids = Vec::with_capacity(optimization_ids.len());
    for chunk in optimization_ids.chunks(400) {
        let proc = ProcedureGetBody::new(chunk.to_vec());
        let response: ProcedureGetResponse<OptimizationRecord> =
            client.get_procedure(proc).await.json().await.unwrap();
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

    let mut molecules = HashMap::with_capacity(ids.len());
    for chunk in ids.chunks(400) {
        let proc = MoleculeGetBody::new(chunk.to_vec());
        let response: MoleculeGetResponse =
            client.get_molecule(&proc).await.json().await.unwrap();
        for molecule in response.data {
            molecules.insert(molecule.id.clone(), molecule);
        }
    }

    let mut got = make_results(results, records, molecule_ids, molecules);

    got.sort_by_key(|g| g.0.clone());
    let got: Vec<_> =
        got.into_iter().map(|(a, b, c)| (a, b, c.len())).collect();

    assert_eq!(got, want);
}
