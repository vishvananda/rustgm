//! Qualify lifetime outcomes independently of the gated frontend source consumer.
#[path = "harness/witness_cases.rs"]
mod cases;
use std::{path::Path, process::Command};

#[test]
fn quantified_input_controls_match_pinned_rustc_objects() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/quantified-inputs-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    for case in cases::CASES {
        // Each typed test removes precisely its own requirement and installs that
        // canonical clause at the declaration/body boundary. Keep that distinction
        // explicit: source-gate rejection does not validate a negative lifetime case.
        assert_eq!(case.source.matches("where for<'a>u8:A<'a,T>").count(), 1);
        assert!(!case.body_source().contains("for<'a>u8:A<'a,T>"));
        let input = directory.join(format!("{}.rs", case.name));
        let object = directory.join(format!("{}.o", case.name));
        std::fs::write(&input, case.source)?;
        let output = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=quantified_input",
                "--emit=obj",
                "--cap-lints=allow",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=yes",
                "-C",
                "debug-assertions=no",
            ])
            .arg(&input)
            .arg("-o")
            .arg(&object)
            .output()?;
        std::fs::write(
            directory.join(format!("{}.stderr", case.name)),
            &output.stderr,
        )?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(!case.accepted)),
            "{}: {}",
            case.name,
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        if case.accepted {
            assert!(output.stderr.is_empty());
            std::fs::remove_file(object)?;
        }
    }
    Ok(())
}
