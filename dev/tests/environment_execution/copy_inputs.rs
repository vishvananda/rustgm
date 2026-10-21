//! Reference Copy classification does not require a completed pointee type.
use super::{checked_sources, native};
use std::path::PathBuf;
const CASES: &[(&str, i32, i32, &str)] = &[
    ("shared-result", 0, 0, ""),
    ("shared-move-result", 0, 0, ""),
    ("mutable-result", 0, 0, ""),
    ("shared-noncopy", 0, 0, ""),
    (
        "mutable-twice",
        1,
        0,
        "local may be uninitialized or moved at this read",
    ),
    (
        "mutable-as-fn",
        1,
        1,
        "trait obligation requires an unambiguous",
    ),
    (
        "shared-escape",
        1,
        0,
        "required lifetime does not outlive its target",
    ),
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/capture-copy-shape/{name}.rs"))
}
#[test]
fn reference_shapes_preserve_move_and_lifetime_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24-copy-shape/source/{}",
        std::process::id(),
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, types, reason) in CASES {
        checked_sources::check_modes(
            &fixture(name),
            &evidence,
            name,
            (rust, [types, rust, rust], reason),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn reference_captures_execute_with_unresolved_pointees() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, rust, _, _) in CASES {
        if rust == 0 {
            let source = std::fs::read_to_string(fixture(name))?;
            native::check_cases(
                "parallel/pa24-copy-shape/native",
                &[(
                    name,
                    source
                        .strip_prefix(native::CHECK)
                        .ok_or("missing check prefix")?,
                    false,
                )],
            )?;
        }
    }
    Ok(())
}
