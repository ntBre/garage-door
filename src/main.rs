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

    let proc = ProcedureGetBody::new(response.ids());
    let mut response: ProcedureGetResponse<TorsionDriveRecord> =
        client.get_procedure(proc).await.json().await.unwrap();
    // only keep the complete records
    response.data.retain(|r| r.status.is_complete());

    println!("{} torsion drive records", response.data.len());

    let optimization_ids = response.optimization_ids();

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

    let mut ids = Vec::with_capacity(optimization_ids.len());
    for chunk in optimization_ids.chunks(info.query_limit) {
        let proc = ProcedureGetBody::new(chunk.to_vec());
        let response: ProcedureGetResponse<OptimizationRecord> =
            client.get_procedure(proc).await.json().await.unwrap();
        ids.extend(response.final_molecules());
    }

    // now you have ANOTHER level of indirection: take the final_molecule ids
    // from this last get_procedure call and query for them

    println!("asking for {} molecules", ids.len());

    let mut molecules = Vec::with_capacity(ids.len());
    for chunk in ids.chunks(info.query_limit) {
        let proc = MoleculeGetBody::new(chunk.to_vec());
        let response: MoleculeGetResponse =
            client.get_molecule(&proc).await.json().await.unwrap();
        molecules.extend(response.data);
    }

    println!("received {} molecules", molecules.len());
}
