use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
struct QueryFilter {
    include: Option<bool>,
    exclude: Option<bool>,
}

#[derive(Serialize)]
struct Data {
    collection: String,
    name: String,
}

#[derive(Serialize)]
pub struct CollectionGetBody {
    meta: QueryFilter,
    data: Data,
}

pub enum CollectionType {
    TorsionDrive,
}

impl From<CollectionType> for String {
    fn from(value: CollectionType) -> Self {
        match value {
            CollectionType::TorsionDrive => String::from("torsiondrivedataset"),
        }
    }
}

impl CollectionGetBody {
    /// Construct a new [CollectionGetBody] with `collection_type` and `name`.
    pub fn new(
        collection_type: CollectionType,
        name: impl Into<String>,
    ) -> Self {
        Self {
            meta: QueryFilter {
                include: None,
                exclude: None,
            },
            data: Data {
                collection: collection_type.into(),
                name: name.into(),
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct Attributes {
    canonical_isomeric_explicit_hydrogen_mapped_smiles: String,
    inchi_key: String,
}

#[derive(Debug, Deserialize)]
pub struct TorsionDriveResult {
    pub name: String,

    // there's only one attribute we care about for now
    attributes: Attributes,

    object_map: HashMap<String, String>,
}

impl TorsionDriveResult {
    /// return the record's id
    pub fn record_id(&self) -> &String {
        assert_eq!(self.object_map.len(), 1);
        self.object_map.get("default").unwrap()
    }

    #[inline]
    pub const fn cmiles(&self) -> &String {
        &self
            .attributes
            .canonical_isomeric_explicit_hydrogen_mapped_smiles
    }

    #[inline]
    pub const fn inchi_key(&self) -> &String {
        &self.attributes.inchi_key
    }
}

/// the important fields in a [CollectionGetResponse]
#[derive(Debug, Deserialize)]
pub struct DataSet {
    pub id: String,
    pub collection: String,
    pub name: String,

    /// the keys are actually smiles strings, but they appear to be roughly the
    /// same as the `name` field on [Record] itself.
    pub records: HashMap<String, TorsionDriveResult>,
}

#[derive(Debug, Deserialize)]
pub struct CollectionGetResponse {
    pub meta: HashMap<String, Value>,
    pub data: Vec<DataSet>,
}

impl CollectionGetResponse {
    pub fn ids(&self) -> Vec<String> {
        self.data
            .iter()
            .flat_map(|ds| ds.records.values())
            .map(|rec| rec.record_id())
            .cloned()
            .collect()
    }
}
