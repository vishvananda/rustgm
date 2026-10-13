#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    resolve::{expansion::Report, model::DefId},
    sema::{
        model::Scalar,
        values::{Constant, Evaluator, LocationId, Origin, Reader, View},
    },
};
use std::{path::Path, process::Command};

const SOURCE: &str = "struct S{p:(u8,[u16;2])}
static A:S=S{p:(3,[7,11])};static B:S=S{p:(3,[7,11])};
const RA:&u16=&A.p.1[1];const RB:&u16=&B.p.1[1];const RE:&u16=&*RA;
const RP:&(u8,[u16;2])=&A.p;const RV:&u16=&RP.1[0];
const LOCAL:&u16=&(3u8,[7u16,11]).1[1];
const BASE:&u16=&11;const AGAIN:&u16=&*BASE;
const REPEAT:[u16;1000000]=[11;1000000];const TEXT:&str=\"hello\";const BYTES:&[u8;3]=b\"abc\";";
fn definition(report: &Report, name: &str) -> Result<DefId, &'static str> {
    report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == name)
        .and_then(|i| u32::try_from(i).ok())
        .map(DefId)
        .ok_or("missing fixture definition")
}
fn reference(reader: Reader<'_>, value: Constant) -> Result<LocationId, &'static str> {
    if let View::Reference(id) = reader.view(value)? {
        Ok(id)
    } else {
        Err("missing constant reference")
    }
}
fn ancestry(
    reader: Reader<'_>,
    mut id: LocationId,
) -> Result<(Option<DefId>, Vec<u64>), &'static str> {
    let mut indices = Vec::new();
    for _ in 0..100 {
        match reader.location(id)?.origin {
            Origin::Allocation => {
                indices.reverse();
                return Ok((None, indices));
            }
            Origin::Static(id) => {
                indices.reverse();
                return Ok((Some(id), indices));
            }
            Origin::Projection { base, index } => {
                indices.push(index);
                id = base;
            }
        }
    }
    Err("cyclic constant projection ancestry")
}
fn contents(reader: Reader<'_>, id: LocationId) -> Result<Constant, &'static str> {
    if let Some(value) = reader.location(id)?.value {
        return Ok(value);
    }
    let (root, path) = ancestry(reader, id)?;
    let mut value = root
        .and_then(|id| reader.definition(id))
        .ok_or("static backing")?;
    for index in path {
        value = reader.field(value, index)?;
    }
    Ok(value)
}
#[test]
fn constant_places_retain_static_identity_and_nested_subobject_paths()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/constant-locations-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let path = directory.join("input.rs");
    std::fs::write(&path, SOURCE)?;
    let independent = Command::new("rustc")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--crate-type=lib",
            "--crate-name=probe",
            "--cap-lints=allow",
            "--emit=metadata",
            "-o",
        ])
        .arg(directory.join("probe.rmeta"))
        .arg(&path)
        .output()?;
    std::fs::write(directory.join("rustc.stderr"), &independent.stderr)?;
    assert!(
        independent.status.success(),
        "{}",
        String::from_utf8_lossy(&independent.stderr)
    );
    let (report, mut facts) = semantic::analyze(SOURCE)?;
    let a = definition(&report, "A")?;
    let b = definition(&report, "B")?;
    let mut completed = Vec::new();
    {
        let mut evaluator = Evaluator::new(&report, &mut facts);
        for name in [
            "RA", "RB", "RE", "RP", "RV", "LOCAL", "BASE", "AGAIN", "REPEAT", "TEXT", "BYTES",
        ] {
            let id = definition(&report, name)?;
            completed.push((name, evaluator.definition(id)?));
        }
        let reader = Reader::new(evaluator.facts());
        assert!(reader.definition(a).is_none() && reader.definition(b).is_none());
        evaluator.complete_static_storage()?;
    }
    let reader = Reader::new(&facts);
    let base = reader
        .definition(definition(&report, "BASE")?)
        .ok_or("base")?;
    let again = reader
        .definition(definition(&report, "AGAIN")?)
        .ok_or("again")?;
    assert_eq!(reference(reader, base)?, reference(reader, again)?);
    for (name, root, path) in [
        ("RA", Some(a), vec![0, 1, 1]),
        ("RB", Some(b), vec![0, 1, 1]),
        ("RE", Some(a), vec![0, 1, 1]),
        ("RP", Some(a), vec![0]),
        ("RV", Some(a), vec![0, 1, 0]),
        ("LOCAL", None, vec![1, 1]),
    ] {
        let value = completed
            .iter()
            .find(|(n, _)| *n == name)
            .ok_or("missing evaluated constant")?
            .1;
        let id = reference(reader, value)?;
        let contents = contents(reader, id)?;
        assert_eq!(ancestry(reader, id)?, (root, path), "{name}");
        if name != "RP" {
            let View::Scalar(Scalar::Integer(bits)) = reader.view(contents)? else {
                return Err("reference leaf is not an integer".into());
            };
            assert_eq!(bits, if name == "RV" { 7 } else { 11 });
        }
    }
    assert!(reader.definition(a).is_some() && reader.definition(b).is_some());
    let repeated = completed
        .iter()
        .find(|(n, _)| *n == "REPEAT")
        .ok_or("repeat")?
        .1;
    let View::Repeat { value, length } = reader.view(repeated)? else {
        return Err("repeat expanded into fields".into());
    };
    assert_eq!(length, 1_000_000);
    assert!(matches!(
        reader.view(value)?,
        View::Scalar(Scalar::Integer(11))
    ));
    assert!(matches!(
        reader.view(reader.field(repeated, 999_999)?)?,
        View::Scalar(Scalar::Integer(11))
    ));
    assert!(reader.field(repeated, 1_000_000).is_err());
    for (name, expected) in [("TEXT", b"hello".as_slice()), ("BYTES", b"abc".as_slice())] {
        let value = completed.iter().find(|(n, _)| *n == name).ok_or("text")?.1;
        let (View::Text(id) | View::Bytes(id)) = reader.view(value)? else {
            return Err("missing interned text".into());
        };
        assert_eq!(reader.bytes(id)?, expected);
    }
    Ok(())
}
