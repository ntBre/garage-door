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

use garage_door::{
    client::FractalClient,
    collection::{CollectionGetBody, CollectionType},
};

#[tokio::main]
async fn main() {
    let col = CollectionGetBody::new(
        CollectionType::TorsionDrive,
        "OpenFF multiplicity correction torsion drive data v1.1",
    );
    let client = FractalClient::new();
    client.retrieve_dataset(col).await;
}
