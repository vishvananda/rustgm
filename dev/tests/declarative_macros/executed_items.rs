//! Original scoped source contracts use reviewed independent outcomes.
//! False original predictions remain in their contracts and observation records.
use super::{fixture, retained_items::lower};
use std::path::Path;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const CASES: &[(&str, Option<&str>)] = &[
    ("concise-parameter", Some("7\n")),
    ("ordered-arms", Some("9\n")),
    ("forward-item", Some("11\n")),
    ("imported-alias", Some("13\n")),
    ("private-definition-helper", None),
    ("captured-caller-expression", Some("17\n")),
    ("opaque-local-does-not-leak", None),
    ("opaque-local-no-capture", None),
    ("semiopaque-caller-item", Some("5\n")),
    ("semiopaque-local-no-capture", None),
    ("transparent-caller-local", Some("19\n")),
    ("restricted-visibility", Some("23\n")),
    ("private-sibling-use", None),
    ("missing-feature", None),
    ("comma-required", None),
    ("legacy-semicolon-control", Some("31\n")),
    ("legacy-comma-rejected", None),
    ("crate-marker", Some("37\n")),
    ("introduced-items-remain-hygienic", Some("82\n")),
    ("legacy-inside-opaque", None),
    ("opaque-inside-legacy", Some("3\n")),
    ("captured-generated-macro-name", Some("17\n")),
    ("captured-module-segment", Some("23\n")),
    ("captured-import-path-alias", Some("31\n")),
    ("public-definition-helper", Some("3\n")),
    ("exported-legacy-inside-opaque", Some("7\n")),
    ("field-literal-roundtrip", Some("7\n")),
    ("field-captured-roundtrip", Some("11\n")),
    ("field-shorthand-local", Some("17\n")),
    ("field-capture-literal-distinct", Some("31\n")),
    ("field-private-definition-access", Some("23\n")),
    ("field-private-captured-denied", None),
    ("field-private-caller-denied", None),
    ("field-functional-update-definition", None),
    ("field-functional-update-caller-denied", None),
    ("field-foreign-root-container", Some("37\n")),
    ("field-foreign-opaque-denied", None),
    ("field-duplicate-declaration", None),
    ("field-duplicate-initializer", None),
    ("field-ordinary-tuple-control", Some("43\n")),
    ("field-ordinary-record-control", Some("47\n")),
    ("field-update-literal-base", Some("29\n")),
    ("field-update-captured-base-denied", None),
    ("field-tuple-pattern-captured-binding", Some("41\n")),
    ("field-tuple-pattern-captured-constructor-denied", None),
    ("field-tuple-access-definition", Some("43\n")),
    ("field-tuple-access-captured-index-denied", None),
    ("field-update-complete-captured-base", Some("29\n")),
    ("field-update-captured-explicit-denied", None),
    ("field-unrelated-container-denied", None),
];
#[test]
fn original_scoped_source_contracts_follow_independently_observed_outcomes() -> TestResult {
    let mut failures = Vec::new();
    for &(name, expected) in CASES {
        let result = lower(&fixture(name)?);
        match (result, expected) {
            (Ok(_), Some(_)) => {}
            (Err(error), None) if !error.to_string().contains("not implemented") => {
                eprintln!("scoped rejection {name}: {error}");
            }
            (Ok(_), None) => failures.push(format!("{name}: invalid scoped source accepted")),
            (Err(error), _) => failures.push(format!("{name}: {error}")),
        }
    }
    if !failures.is_empty() {
        return Err(failures.join("\n").into());
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn original_scoped_source_contracts_execute_the_observed_native_answers() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/scoped-execution-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let consumer =
        std::fs::read_to_string(root.join("pa31/tests/spec/source-panic-macros/consumer.c"))?;
    for &(name, expected) in CASES {
        let Some(expected) = expected else { continue };
        let program = lower(&fixture(name)?).map_err(|error| format!("{name}: {error}"))?;
        crate::ffi_program::check(root, &folder, name, &program, &consumer, expected)?;
    }
    Ok(())
}
