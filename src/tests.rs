use std::fs::read_to_string;

use openff_toolkit::qcsubmit::results::TorsionDriveResultCollection;

use crate::{
    client::FractalClient,
    collection::{CollectionGetBody, CollectionGetResponse, CollectionType},
    procedure::{OptimizationRecord, Response, TorsionDriveRecord},
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
    let c: Response<TorsionDriveRecord> = serde_json::from_str(&s).unwrap();
    dbg!(c);
}

#[test]
fn de_opt_procedure() {
    let s = read_to_string("testfiles/opt_procedure.json").unwrap();
    let c: Response<OptimizationRecord> = serde_json::from_str(&s).unwrap();
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
        CollectionType::TorsionDrive,
        "OpenFF multiplicity correction torsion drive data v1.1",
    );

    let mut got = client
        .retrieve_dataset(col, CollectionType::TorsionDrive)
        .await;

    got.sort_by_key(|g| g.0.clone());
    let got: Vec<_> =
        got.into_iter().map(|(a, b, c)| (a, b, c.len())).collect();

    assert_eq!(got, want);
}

#[tokio::test]
async fn full_opt() {
    let want = {
        let s = read_to_string("testfiles/final_opt.dat").unwrap();
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
    let ds =
        TorsionDriveResultCollection::parse_file("testfiles/core-opt.json")
            .unwrap();
    let col: CollectionGetResponse = ds.into();
    let query_limit = client.get_query_limit().await;
    let mut got = client
        .to_records(col, query_limit, CollectionType::Optimization)
        .await;

    got.sort_by_key(|g| g.0.clone());
    // NOTE: unlike above, comparing the length of the geometry (in atoms)
    // rather than the length of the conformers vector because it should always
    // contain a single conformer
    let got: Vec<_> = got
        .into_iter()
        .map(|(a, b, c)| (a, b, c[0].len() / 3))
        .collect();

    assert_eq!(got.len(), want.len());

    for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
        assert_eq!(g, w, "mismatch at {i}: got:\n{g:#?}\nwant:\n{w:#?}\n");
    }
    assert_eq!(got, want);
}
