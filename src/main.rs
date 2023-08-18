use std::collections::HashMap;

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
    let col = CollectionGetBody::new(
        "torsiondrivedataset",
        "OpenFF multiplicity correction torsion drive data v1.1",
    );
    let client = FractalClient::new();
    let info = client.get_information().await.unwrap();
    println!("query_limit = {}", info.query_limit);
    let response: CollectionGetResponse =
        client.get_collection(col).await.json().await.unwrap();

    dbg!(response.ids().contains(&"97609630".to_owned()));

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

    println!("{} torsion drive records", records.data.len());

    let optimization_ids = records.optimization_ids();
    dbg!(optimization_ids.contains(&"97609630".to_owned()));

    // the goal is to replicate the sequence of Python code:
    //
    // ```python
    // client = FractalClient()
    // collection = TorsionDriveResultCollection.from_server(
    //     client=client,
    //     datasets=[
    //         "OpenFF multiplicity correction torsion drive data v1.1",
    //     ],
    //     spec_name="default",
    // )
    // records_and_molecules = collection.to_records()
    // ```
    //
    // so far, I can construct the client and retrieve the collection from the
    // server. and I'm in the middle of calling to_records. the actual record
    // part is easy enough: just the records from the initial call

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

    // this is a map of (record_id, grid_id) -> opt_record_id
    let mut molecule_ids = HashMap::new();

    let mut ids = Vec::with_capacity(optimization_ids.len());
    for chunk in optimization_ids.chunks(info.query_limit) {
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

    // ids somehow contains the value I need to insert into molecule_ids below,
    // but how in the world do I match them up? I guess they're in the same
    // order?
    dbg!(ids.contains(&"97609630".to_owned()));

    // now you have ANOTHER level of indirection: take the final_molecule ids
    // from this last get_procedure call and query for them

    println!("asking for {} molecules", ids.len());

    let mut molecules = HashMap::with_capacity(ids.len());
    for chunk in ids.chunks(info.query_limit) {
        let proc = MoleculeGetBody::new(chunk.to_vec());
        let response: MoleculeGetResponse =
            client.get_molecule(&proc).await.json().await.unwrap();
        for molecule in response.data {
            molecules.insert(molecule.id.clone(), molecule);
        }
    }
    dbg!(molecules
        .keys()
        .collect::<Vec<_>>()
        .contains(&&"97609630".to_owned()));

    println!("received {} molecules", molecules.len());

    // at this point I have all of the TorsionDriveRecords and all of the
    // QCMolecules. these can be transformed to OpenFF Molecules in
    // _cached_query_single_structure_results, which builds a Molecule from the
    // record.cmiles and then adds the corresponding QCMolecule.geometries as
    // conformers. Potentially the last thing for me to do (before rodeo can
    // generate Molecules like rdkit) is to map the molecules back to their
    // corresponding torsions

    // there may be more results than records, but accessing them with this map
    // by the id stored on the records ensures that I only get the ones I want
    let cmiles_map: HashMap<_, _> = results
        .iter()
        .map(|rec| (rec.record_id(), rec.cmiles()))
        .collect();

    for record in &records.data {
        let mut grid_ids: Vec<_> = record.minimum_positions.keys().collect();
        grid_ids.sort_by_key(|g| {
            let x: &[_] = &['[', ']'];
            g.trim_matches(x).parse::<isize>().unwrap()
        });

        let mut qc_grid_molecules = Vec::new();
        for grid_id in &grid_ids {
            let i = &molecule_ids[&(record.id.clone(), (*grid_id).clone())];
            qc_grid_molecules.push(molecules[i].clone());
        }

        // need to return Vec<(record, cmiles, Vec<Geometry>)> as described
        // above. The record is passed along directly; cmiles is used to
        // construct the initial molecule; and the geometries are used to add
        // conformers to the Molecule
        println!(
            "{} => {} => {}",
            record.id,
            cmiles_map[&record.id],
            qc_grid_molecules.len()
        );
    }
}
