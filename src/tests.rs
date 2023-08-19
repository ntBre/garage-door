use std::fs::read_to_string;

use crate::{
    client::FractalClient,
    collection::{CollectionGetBody, CollectionGetResponse},
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

#[tokio::test]
async fn full() {
    let want = {
        let s = read_to_string("testfiles/final.dat").unwrap();
        let lines = s.lines();
        let mut ret = Vec::new();
        for line in lines {
            let sp: Vec<_> = line.split("=>").map(|s| s.trim()).collect();
            ret.push((
                sp[0].to_owned(),
                sp[1].to_owned(),
                sp[2].parse::<usize>().unwrap(),
            ));
        }
        ret.sort_by_key(|r| r.0.clone());
        ret
    };

    let client = FractalClient::new();
    let col = CollectionGetBody::new(
        "torsiondrivedataset",
        "OpenFF multiplicity correction torsion drive data v1.1",
    );

    let mut got = client.retrieve_dataset(col).await;

    got.sort_by_key(|g| g.0.clone());
    let got: Vec<_> =
        got.into_iter().map(|(a, b, c)| (a, b, c.len())).collect();

    assert_eq!(got, want);
}
