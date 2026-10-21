//! Assigned public placeholders preserve their own binder without capturing caller holes.
use super::{checked_sources, native};
use std::path::PathBuf;
const POSITIVES: &[&str] = &["repeated-quantified-functions", "late-object-requirements"];
const NEGATIVES: &[&str] = &[
    "outer-result-cannot-be-local",
    "owned-local-escape",
    "unowned-annotated-input",
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../pa24/tests/spec/signature-slot-ownership/{name}.rs"
    ))
}
#[test]
fn public_slot_ownership_preserves_independent_source_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24-signature-owners/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &name in POSITIVES {
        checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for &name in NEGATIVES {
        checked_sources::check_modes(
            &fixture(name),
            &evidence,
            name,
            (
                1,
                [0, 1, 1],
                "required lifetime does not outlive its target",
            ),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn independent_public_frames_execute_after_repeated_constraints()
-> Result<(), Box<dyn std::error::Error>> {
    for &name in POSITIVES {
        let source = std::fs::read_to_string(fixture(name))?;
        native::check_cases(
            "parallel/pa24-signature-owners/native",
            &[(
                name,
                source
                    .strip_prefix(native::CHECK)
                    .ok_or("missing check prefix")?,
                false,
            )],
        )?;
    }
    Ok(())
}
