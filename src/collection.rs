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

impl CollectionGetBody {
    pub fn new(collection: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            meta: QueryFilter {
                include: None,
                exclude: None,
            },
            data: Data {
                collection: collection.into(),
                name: name.into(),
            },
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

#[derive(Debug, Deserialize)]
pub struct Record {
    pub name: String,

    // TODO this shouldn't be pub, at least in this form. I'd rather deserialize
    // this in a better way. so far, it looks like the key is always "default"
    // and the value is the record I want to request
    object_map: HashMap<String, String>,
}

impl Record {
    /// return the record's id
    pub fn id(&self) -> &String {
        assert_eq!(self.object_map.len(), 1);
        self.object_map.get("default").unwrap()
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
    pub records: HashMap<String, Record>,
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
            .map(|rec| rec.id())
            .cloned()
            .collect()
    }
}
