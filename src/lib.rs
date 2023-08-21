use std::collections::HashMap;

use collection::TorsionDriveResult;
use molecule::Molecule;
use procedure::TorsionDriveRecord;
use serde::{Deserialize, Serialize};

pub mod client;
pub mod collection;
pub mod molecule;
pub mod procedure;

#[cfg(test)]
mod tests;

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

/// constructs output usable by qcsubmit. Returns a vector of (record_id,
/// cmiles, Vec<geometry>), where a geometry is a Vec<f64> to be inserted in a
/// Molecule._conformers. There's not actually code in qcsubmit to do this
/// directly, but see results/caching.py:cached_query_torsion_drive_results for
/// how to reconstruct its output
pub fn make_td_results(
    results: Vec<TorsionDriveResult>,
    records: Vec<TorsionDriveRecord>,
    molecule_ids: HashMap<(String, String), String>,
    molecules: HashMap<String, Molecule>,
) -> Vec<(String, String, Vec<Vec<f64>>)> {
    // there may be more results than records, but accessing them with this map
    // by the id stored on the records ensures that I only get the ones I want
    let cmiles_map: HashMap<_, _> = results
        .iter()
        .map(|rec| (rec.record_id(), rec.cmiles()))
        .collect();

    let mut ret = Vec::new();
    for record in records {
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

        let cmiles = cmiles_map[&record.id].clone();
        ret.push((
            record.id,
            cmiles,
            qc_grid_molecules.into_iter().map(|m| m.geometry).collect(),
        ));
    }

    ret
}

/// Analagous to [make_td_results] but without all of the bookkeeping mapping
/// individual molecules back to their corresponding TorsionDrives. Just pass in
/// a Vec<Molecule> and get back a Vec<(id, cmiles, Vec<Geometry>)>. The
/// Vec<Geometry> will always have length one. A vector is used just to keep the
/// return type consistent with the TorsionDrive version.
pub fn make_opt_results(
    results: Vec<TorsionDriveResult>,
    records: Vec<Molecule>,
    molecule_ids: HashMap<String, String>,
) -> Vec<(String, String, Vec<Vec<f64>>)> {
    // there may be more results than records, but accessing them with this map
    // by the id stored on the records ensures that I only get the ones I want
    let cmiles_map: HashMap<_, _> = results
        .iter()
        .map(|rec| (rec.record_id(), rec.cmiles()))
        .collect();

    let mut ret = Vec::new();
    for record in records {
        // do this first so we don't have to clone record.id
        let id = &molecule_ids[&record.id];
        let cmiles = cmiles_map[id].clone();
        ret.push((id.clone(), cmiles, vec![record.geometry]));
    }

    ret
}
