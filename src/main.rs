use client::FractalClient;
use procedure::ProcedureGetBody;

use collection::CollectionGetBody;

use crate::{
    collection::CollectionGetResponse, procedure::ProcedureGetResponse,
};

mod client;
mod collection;
mod procedure;

#[cfg(test)]
mod tests;

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
