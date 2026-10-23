//! Explicit pattern modifiers retain the edition of their parsed introducer.
use rustgm::support::edition::Edition;
use std::path::Path;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CASES: &[(&str, bool, bool)] = &[
    ("plain", true, true),
    ("mut-reset", true, false),
    ("ref-shared", true, false),
    ("ref-mut", true, false),
    ("ref-mut-shared", false, false),
    ("reference-reset", true, false),
    ("explicit-outer", true, true),
    ("at-binding", true, false),
    ("macro-generated", true, false),
    ("opaque-forwarded", true, false),
    ("shorthand", true, false),
    ("reference-tuple-reset", true, false),
    ("noncopy-move", false, false),
];

fn source(name: &str) -> Result<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    Ok(std::fs::read_to_string(
        root.join("pa34/tests/spec/pattern-editions")
            .join(format!("{name}.rs")),
    )?)
}

#[test]
fn paired_binding_modes_reach_the_complete_typed_pipeline() -> Result {
    for &(name, older, newer) in CASES {
        let source = source(name)?;
        for (edition, accepted) in [(Edition::Rust2021, older), (Edition::Rust2024, newer)] {
            let result = crate::edition_packages::lowir(&source, edition);
            match result {
                Ok(bytes) => assert!(accepted && !bytes.is_empty(), "{name}/{edition:?}"),
                Err(error) => {
                    assert!(!accepted, "{name}/{edition:?}: {error}");
                    let error = error.to_string();
                    if edition == Edition::Rust2024 {
                        assert!(
                            error.contains("default binding mode in edition 2024"),
                            "{error}"
                        );
                    } else {
                        assert!(
                            error.contains("mutab")
                                || error.contains("shared")
                                || error.contains("borrow")
                                || error.contains("move"),
                            "{error}"
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires pinned rustc and CPPGM native backend"]
fn binding_values_and_mutations_match_each_selected_edition() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    let folder = root.join("obj/parallel/pa34-pattern-native").join(format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(name, older, newer) in CASES {
        for (edition, accepted) in [(Edition::Rust2021, older), (Edition::Rust2024, newer)] {
            if accepted {
                crate::edition_packages::native(&folder, name, &source(name)?, edition, root)?;
            }
        }
    }
    Ok(())
}
