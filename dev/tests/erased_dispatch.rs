//! Erased callables preserve dynamic targets, reference prefixes and the Rust ABI.
#[path = "harness/checked_sources.rs"]
mod checked_sources;
#[path = "harness/native_references.rs"]
mod native;
use std::path::PathBuf;

const CASES: &[&str] = &[
    "item",
    "pointer",
    "empty",
    "mutable",
    "generic-item",
    "aggregate",
    "generic-consumer",
    "borrowed-consumer",
    "inherited",
    "upcast",
    "higher-ranked",
    "static",
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/callable-dispatch/{name}.rs"))
}
#[test]
fn callable_object_views_match_declared_objects() -> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/callable-dispatch/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &name in CASES {
        checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM backend; run make test-maintainer-native"]
fn callable_objects_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &name in CASES {
        let source = std::fs::read_to_string(fixture(name))?;
        let body = source
            .strip_prefix(native::CHECK)
            .ok_or("missing check prefix")?;
        native::check_cases(
            "parallel/pa24/callable-dispatch/native",
            &[(name, body, false)],
        )?;
    }
    Ok(())
}
