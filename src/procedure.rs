use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub enum Status {
    #[serde(rename = "COMPLETE")]
    Complete,
    #[serde(rename = "ERROR")]
    Error,
}

#[derive(Default, Serialize)]
struct QueryFilter {
    include: Option<usize>,
    exclude: Option<usize>,
    limit: Option<usize>,
    skip: usize,
}

#[derive(Serialize)]
struct Data {
    id: Vec<String>,
    task_id: Option<usize>,
    procedure: Option<usize>,
    program: Option<usize>,
    hash_index: Option<usize>,
    status: Status,
}

#[derive(Serialize)]
pub struct ProcedureGetBody {
    meta: QueryFilter,
    data: Data,
}

impl ProcedureGetBody {
    pub fn new(id: Vec<String>) -> Self {
        Self {
            meta: QueryFilter::default(),
            data: Data {
                id,
                task_id: None,
                procedure: None,
                program: None,
                hash_index: None,
                status: Status::Complete,
            },
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

#[derive(Debug, Deserialize)]
pub struct OptimizationSpec {
    pub program: String,
    pub keywords: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    pub id: String,
    pub initial_molecule: Vec<String>,
    pub optimization_spec: OptimizationSpec,

    #[serde(rename = "final_energy_dict")]
    pub final_energies: HashMap<String, f64>,

    pub minimum_positions: HashMap<String, f64>,

    pub status: Status,
}

#[derive(Debug, Deserialize)]
pub struct ProcedureGetResponse {
    pub meta: Value,
    pub data: Vec<Record>,
}
