use garage_door::{
    client::FractalClient,
    collection::{CollectionGetBody, CollectionGetResponse},
    procedure::{ProcedureGetBody, ProcedureGetResponse},
};

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

    let optimization_ids = response.optimization_ids();

    println!(
        "{}",
        client
            .get_procedure(ProcedureGetBody::new(optimization_ids))
            .await
            .text()
            .await
            .unwrap()
    );
}
