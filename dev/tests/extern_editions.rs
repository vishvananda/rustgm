//! Foreign edition syntax preserves actual call permission and exact linkage.
use rustgm::support::edition::Edition;
use std::path::Path;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CASES: &[(&str, bool, bool)] = &[
    ("plain-block", true, false),
    ("unsafe-block", true, true),
    ("safe-member", true, true),
    ("plain-safe-member", false, false),
    ("plain-unsafe-member", false, false),
    ("plain-safe-static", false, false),
    ("plain-unsafe-static", false, false),
    ("plain-call-without-permission", false, false),
    ("plain-static", true, false),
    ("bare-no-mangle", true, false),
    ("bare-export-name", true, false),
    ("wrapped-export-name", true, true),
    ("wrapped-link-name", false, false),
    ("macro-name", true, false),
    ("forwarded-meta", true, false),
    ("macro-block", true, false),
];

fn source(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa34/tests/spec/foreign-editions")
            .join(format!("{name}.rs")),
    )?)
}

#[test]
fn foreign_syntax_uses_its_edition_without_granting_call_permission() -> Result {
    for &(name, older, newer) in CASES {
        let source = source(name)?;
        for (edition, accepted) in [(Edition::Rust2021, older), (Edition::Rust2024, newer)] {
            let result = crate::edition_packages::lowir(&source, edition);
            match result {
                Ok(bytes) => assert!(accepted && !bytes.is_empty(), "{name}/{edition:?}"),
                Err(error) => {
                    assert!(!accepted, "{name}/{edition:?}: {error}");
                    let error = error.to_string();
                    assert!(error.contains("unsafe"), "{name}/{edition:?}: {error}");
                }
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires pinned rustc and CPPGM native backend"]
fn foreign_calls_and_exports_execute_in_their_defining_edition() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    let folder = root.join("obj/parallel/pa34-foreign-native").join(format!(
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
