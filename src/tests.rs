use std::fs::read_to_string;

use crate::{
    collection::CollectionGetResponse,
    procedure::{OptimizationRecord, ProcedureGetResponse, TorsionDriveRecord},
};

#[test]
fn de_response() {
    let s = read_to_string("testfiles/response.json").unwrap();
    let c: CollectionGetResponse = serde_json::from_str(&s).unwrap();
    dbg!(c);
}

#[test]
fn de_procedure() {
    let s = read_to_string("testfiles/procedure.json").unwrap();
    let c: ProcedureGetResponse<TorsionDriveRecord> =
        serde_json::from_str(&s).unwrap();
    dbg!(c);
}

#[test]
fn de_opt_procedure() {
    let s = read_to_string("testfiles/opt_procedure.json").unwrap();
    let c: ProcedureGetResponse<OptimizationRecord> =
        serde_json::from_str(&s).unwrap();
    dbg!(c);
}
