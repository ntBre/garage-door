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

use clap::{Parser, Subcommand};
use garage_door::{
    client::FractalClient,
    collection::{CollectionGetBody, CollectionGetResponse, CollectionType},
};
use openff_toolkit::qcsubmit::results::TorsionDriveResultCollection;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Retrieve a named dataset from QCArchive and convert it to a series of
    /// records and molecules
    Get {
        /// The type of dataset to be retrieved
        #[arg(short, long)]
        dataset_type: CollectionType,

        /// Data set name to retrieve
        name: String,
    },

    /// Convert an existing data set to a series of records and molecules
    Convert {
        /// The type of dataset to be retrieved
        #[arg(short, long)]
        dataset_type: CollectionType,

        /// JSON file containing the data set
        filename: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let client = FractalClient::new();
    let results = match args.command {
        Commands::Get { name, dataset_type } => {
            let col = CollectionGetBody::new(dataset_type, name);
            client.retrieve_dataset(col, dataset_type).await
        }
        Commands::Convert {
            filename,
            dataset_type,
        } => {
            // as I found out, you can always parse from file as a td collection
            let ds =
                TorsionDriveResultCollection::parse_file(filename).unwrap();
            let col: CollectionGetResponse = ds.into();
            let query_limit = client.get_query_limit().await;
            client.to_records(col, query_limit, dataset_type).await
        }
    };

    let s = serde_json::to_string_pretty(&results);
    if s.is_err() {
        eprintln!("error serializing result to JSON. dumping what we can");
        println!("{:#?}", results);
    }
    println!("{}", s.unwrap());
}
