//! Slice len selects one checked canonical method, including qualified and const use.
use super::{mutation::frontend, native, rustc};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool, bool)] = &[
    ("array-local", true, true),
    ("slice-local", true, true),
    ("array-qualified", false, false),
    ("slice-qualified", true, true),
    ("array-reified", false, false),
    ("slice-reified", true, true),
    ("array-alias", false, false),
    ("slice-alias", true, true),
    ("generic-array", true, true),
    ("generic-slice", true, true),
    ("const-array", true, true),
    ("const-slice", true, true),
    ("const-generic-array", true, true),
    ("const-generic-slice", true, true),
    ("const-qualified", true, true),
    ("mutable-slice", true, true),
    ("nested-references", true, true),
    ("noncopy-elements", true, true),
    ("shorter-slice", true, true),
    ("no-prelude", true, false),
    ("local-inherent", true, true),
    ("trait-array-competition", true, true),
    ("trait-slice-competition", true, true),
    ("generic-trait-lookalike", true, true),
    ("uninitialized-array", false, false),
    ("uninitialized-slice", false, false),
    ("active-mut-loan", false, false),
    ("active-element-loan", false, false),
    ("wrong-argument", false, false),
    ("wrong-reified-input", false, false),
    ("fn-pointer-const", false, false),
    ("mutable-static-unsafe", false, false),
    ("mutable-static-lint", false, false),
    ("array-qualified-trait", true, true),
    ("array-alias-trait", true, true),
    ("dot-turbofish", false, false),
    ("qualified-turbofish", false, false),
    ("named-pattern-metadata", true, true),
    ("invalid-surrounding", false, false),
    ("named-empty-metadata", true, true),
    ("named-zst-metadata", true, true),
    ("generic-array-lifetimes", true, true),
    ("generic-array-wrong-element", false, false),
    ("generic-array-unpaid-lifetime", false, false),
    ("generic-array-shared-covariance", true, true),
    ("generic-array-mutable-invariance", false, false),
    ("generic-array-mutable-shortening", true, true),
    ("byte-pattern-metadata", true, true),
    ("empty-byte-pattern", true, true),
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../pa26/tests/spec/slice-length/{name}.rs"))
}
#[test]
fn declared_slice_lengths() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, _) in CASES {
        let source = std::fs::read_to_string(fixture(name))?;
        rustc::qualify(
            &format!("parallel/pa26/slice-length/source/{name}"),
            &source,
            accepted,
        )?;
        let result = frontend::checked(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn selected_slice_lengths_execute() -> Result<(), Box<dyn std::error::Error>> {
    let sources = CASES
        .iter()
        .filter(|c| c.1 && c.2)
        .map(|c| Ok((c.0, std::fs::read_to_string(fixture(c.0))?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, source)| {
            Ok((
                *name,
                source
                    .strip_prefix(native::CHECK)
                    .ok_or("missing native check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/slice-length/native", &cases)
}

const RETAINED: &[&str] = &[
    "pa25/tests/spec/rest-views/empty-slice.rs",
    "pa25/tests/spec/rest-views/static-length.rs",
    "pa25/tests/spec/rest-views/self-static-length.rs",
    "pa27/tests/spec/rest-views/mutable-static-length.rs",
    "pa28/tests/spec/rest-views/interior-static-length.rs",
    "pa30/tests/spec/rest-views/foreign-static-length.rs",
];
#[test]
fn retained_rest_length_positives() -> Result<(), Box<dyn std::error::Error>> {
    for &path in RETAINED {
        let source =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(path))?;
        rustc::qualify(
            &format!("parallel/pa26/slice-length/retained/{path}"),
            &source,
            true,
        )?;
        frontend::checked(&source).map_err(|e| format!("{path}: {e}"))?;
    }
    Ok(())
}
