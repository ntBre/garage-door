//! [FractalClient] queries for procedures like [TorsionDrive]s and
//! [Optimization]s.

use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub enum Status {
    #[serde(rename = "COMPLETE")]
    Complete,
    #[serde(rename = "ERROR")]
    Error,
}

impl Status {
    /// Returns `true` if the status is [`Complete`].
    ///
    /// [`Complete`]: Status::Complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }
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
pub struct TorsionDriveRecord {
    pub id: String,
    pub initial_molecule: Vec<String>,
    pub optimization_spec: OptimizationSpec,

    #[serde(rename = "final_energy_dict")]
    pub final_energies: HashMap<String, f64>,

    pub minimum_positions: HashMap<String, usize>,

    pub status: Status,

    /// A map of grid points to additional ids
    /// Example:
    /// ```json
    /// "optimization_history": {
    ///   "[-120]": [
    ///     "104321688",
    ///     "104405676",
    ///     "104405677"
    ///   ]
    /// }
    /// ```
    pub optimization_history: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct OptimizationRecord {
    pub id: String,
    pub initial_molecule: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcedureGetResponse<T> {
    pub meta: Value,
    pub data: Vec<T>,
}

impl ProcedureGetResponse<TorsionDriveRecord> {
    pub fn optimization_ids(&self) -> Vec<String> {
        let mut ret = Vec::new();
        for record in &self.data {
            for (grid_id, minimum_idx) in &record.minimum_positions {
                ret.push(
                    record.optimization_history[grid_id][*minimum_idx].clone(),
                );
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn check_opt_ids() {
        let s = read_to_string("testfiles/procedure.json").unwrap();
        let mut c: ProcedureGetResponse<TorsionDriveRecord> =
            serde_json::from_str(&s).unwrap();
        c.data.retain(|f| f.status.is_complete());
        let mut got_ids = c.optimization_ids();
        got_ids.sort();
        got_ids.dedup();
        let want_ids =
            read_to_string("testfiles/optimization_ids.txt").unwrap();
        let want_ids: Vec<&str> = want_ids.split_ascii_whitespace().collect();
        assert_eq!(got_ids, want_ids);
    }
}
