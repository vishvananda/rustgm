//! Named-record metadata is produced and consumed through the checked public driver.
use super::{artifacts, crates};

const ACCEPTED: [&str; 9] = [
    "pass-construct",
    "pass-distinct-libraries",
    "pass-local-wrapper",
    "pass-nested",
    "pass-pattern",
    "pass-private-value",
    "pass-private-wildcard",
    "pass-return",
    "pass-static-field",
];
const REJECTED: [(&str, &str, &str); 6] = [
    ("reject-distinct-nominal", "incompatible types", "E0308"),
    ("reject-moved-nested", "uninitialized at this read", "E0382"),
    ("reject-moved-record", "uninitialized at this read", "E0382"),
    ("reject-private-construct", "unresolved", "E0603"),
    ("reject-private-field", "private", "E0616"),
    ("reject-private-pattern", "private", "E0451"),
];

#[test]
fn closed_record_exports_and_clients_preserve_layout_privacy_and_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let build = artifacts::build_records("record-sources")?;
    build.qualify()?;
    for path in [&build.first, &build.second] {
        let bytes = std::fs::read(path)?;
        let interface = crates::decode(&bytes)?;
        assert_eq!(bytes[12], 3);
        assert_eq!(crates::encode(&interface)?, bytes);
        let nominals = interface.nominals().ok_or("record graph")?;
        assert_eq!(nominals.records().len(), 5);
        assert_eq!(
            nominals
                .records()
                .iter()
                .filter(|record| !record.public())
                .count(),
            1
        );
        assert!(interface.functions().iter().all(|function| {
            function
                .signature()
                .types()
                .shares_storage(nominals.types())
        }));
    }
    for name in ACCEPTED {
        build.qualify_client(name, None)?;
        let first = artifacts::checked(name, &build.folder, &mut build.client(name, false))?;
        let reordered = artifacts::checked(
            &format!("{name}.reordered"),
            &build.folder,
            &mut build.client(name, true),
        )?;
        assert_eq!(first.stdout, reordered.stdout);
    }
    for (name, reason, code) in REJECTED {
        build.qualify_client(name, Some(code))?;
        let output = build.client(name, false).output()?;
        std::fs::write(build.folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert_eq!(output.status.code(), Some(1), "{name}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(reason),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let repeated = artifacts::build_records("record-repeat")?;
    for (left, right) in [
        (&build.first, &repeated.first),
        (&build.second, &repeated.second),
    ] {
        assert_eq!(std::fs::read(left)?, std::fs::read(right)?);
    }
    Ok(())
}

#[test]
#[ignore = "requires configured pinned CPPGM object backend and Rust toolchain"]
fn closed_record_libraries_execute_through_the_shared_backend()
-> Result<(), Box<dyn std::error::Error>> {
    let build = artifacts::build_records("record-native")?;
    build.qualify()?;
    for name in ACCEPTED {
        build.execute(name)?;
    }
    Ok(())
}

#[test]
fn standalone_records_export_and_unsupported_semantic_contracts_stay_explicit()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::resolve::resolve("pub struct Standalone{pub x:u8}", true)?;
    let mut analysis = rustgm::pipeline::analyze_source(&report)?;
    let exported = rustgm::sema::crates::export(
        &report,
        &mut analysis.facts,
        crates::CrateIdentity::new("standalone", "one")?,
    )?;
    assert!(exported.interface.functions().is_empty());
    assert_eq!(exported.interface.records().len(), 1);
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa29/tests/spec/closed-record-boundaries/repr-c.rs"),
    )?;
    let (_, interface) = super::exports::compile(&text, "repr_c")?;
    let decoded = crates::decode(&crates::encode(&interface)?)?;
    assert_eq!(decoded.records().len(), 1);
    assert_eq!(decoded.records()[0].repr(), crates::RecordRepr::C);
    for name in [
        "copy-clone",
        "drop",
        "repr-align",
        "non-exhaustive",
        "lifetime-generic",
        "mutable-field",
        "inherent-method",
        "private-field-type",
        "private-result-type",
    ] {
        let text = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            format!("../pa29/tests/spec/closed-record-boundaries/{name}.rs"),
        ))?;
        let report = crate::resolve::resolve(&text, true)?;
        let mut analysis = rustgm::pipeline::analyze_source(&report)?;
        let result = rustgm::sema::crates::export(
            &report,
            &mut analysis.facts,
            crates::CrateIdentity::new("boundary", "one")?,
        );
        let Err(error) = result else {
            return Err(format!("dropped unsupported contract: {text}").into());
        };
        assert!(error.to_string().contains("profile"), "{text}: {error}");
        if name == "non-exhaustive" {
            assert_eq!(
                error.to_string(),
                "file 0: 0: nominal trait, member or non-exhaustive contract is outside the metadata profile"
            );
        }
    }
    Ok(())
}
