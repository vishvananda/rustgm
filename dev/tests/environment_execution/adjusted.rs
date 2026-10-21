//! Adjusted places retain their typed source prefix and ordinary receiver loans.
use super::{native, rustc, semantic};
const CASES: &[(&str, bool)] = &[
    ("method-shared", true),
    ("method-deep", true),
    ("method-mutable", true),
    ("method-moving", true),
    ("method-field", true),
    ("call-function", true),
    ("call-closure", true),
    ("call-mutable", true),
    ("method-conflict", false),
    ("method-shared-mutation", false),
];
fn fixture(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/adjusted-places/{name}.rs")),
    )
}
#[test]
fn declared_adjusted_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/adjusted-execution/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let source = fixture(name)?;
        rustc::qualify(
            &format!("parallel/pa24/adjusted-execution/qualification/{name}"),
            &source,
            accepted,
        )?;
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, &source)?;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&path)
            .output()?;
        std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
        if output.status.code() != Some(i32::from(!accepted)) {
            failures.push(format!(
                "{name}: {:?}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn selected_receiver_cannot_skip_a_larger_capture_prefix() -> Result<(), Box<dyn std::error::Error>>
{
    let (report, mut facts) = semantic::analyze(&fixture("method-deep")?)?;
    let selected = facts
        .methods
        .calls
        .values_mut()
        .next()
        .ok_or("missing method")?;
    assert_eq!(selected.dereferences, 2);
    selected.dereferences = 1;
    let error = rustgm::body::build_concrete(&report, &facts)
        .err()
        .ok_or("mismatched receiver prefix was accepted")?;
    assert!(
        error
            .to_string()
            .contains("capture prefix exceeds selected receiver dereferences")
    );
    Ok(())
}
#[test]
fn native_adjusted_receivers() -> Result<(), Box<dyn std::error::Error>> {
    let sources = CASES
        .iter()
        .filter(|case| case.1)
        .map(|&(name, _)| fixture(name).map(|source| (name, source)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut failures = Vec::new();
    for (name, source) in sources {
        if let Err(error) = native::check_cases(
            "parallel/pa24/adjusted-execution/native",
            &[(name, &source, false)],
        ) {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn value_fixture(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/adjusted-native/{name}.rs")),
    )
}
#[test]
fn independent_adjusted_values() -> Result<(), Box<dyn std::error::Error>> {
    for name in ["deep-reference", "mutable-call"] {
        rustc::qualify(
            &format!("parallel/pa24/adjusted-execution/qualification/{name}"),
            &value_fixture(name)?,
            true,
        )?;
    }
    Ok(())
}
#[test]
fn native_adjusted_values() -> Result<(), Box<dyn std::error::Error>> {
    for name in ["deep-reference", "mutable-call"] {
        native::check_cases(
            "parallel/pa24/adjusted-execution/values",
            &[(name, &value_fixture(name)?, false)],
        )?;
    }
    Ok(())
}
