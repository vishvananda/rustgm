//! Qualified expression paths prepare exact nominal facts without defeating shadowing.
use super::{root, semantic};
use std::process::Command;
#[test]
fn qualified_atomic_value_preparation() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::support::library_types::LibraryType;
    for kind in LibraryType::ALL {
        // Unqualified names are preparation hints; a module proves exact identity.
        let hint = LibraryType::ALL
            .into_iter()
            .find(|candidate| candidate.name() == kind.name());
        assert_eq!(LibraryType::from_name(kind.name()), hint);
        for other in LibraryType::ALL {
            if kind != other && kind.name() == other.name() {
                assert_ne!(kind.module(), other.module());
            }
        }
    }
    let root = root()?;
    let output = root.join(format!(
        "obj/parallel/pa28/atomic-preparation-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&output)?;
    for (name, canonical) in [
        ("qualified", true),
        ("extern-qualified", true),
        ("no-prelude-qualified", true),
        ("absolute", true),
        ("core-alias", true),
        ("new-only", true),
        ("load-only", true),
        ("ordering-only", true),
        ("local-core", false),
        ("local-types", false),
        ("ordinary", false),
    ] {
        let path = root.join(format!("pa28/tests/spec/atomic-preparation/{name}.rs"));
        let text = std::fs::read_to_string(&path)?;
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=probe",
                "--emit=obj",
                "-C",
                "panic=abort",
                "-o",
            ])
            .arg(output.join(format!("{name}.o")))
            .arg(&path)
            .output()?;
        std::fs::write(
            output.join(format!("{name}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert!(
            independent.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        for mode in ["--emit-types", "--emit-lowir", "--emit-constants"] {
            let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&path)
                .output()?;
            std::fs::write(output.join(format!("{name}.{mode}.stdout")), &result.stdout)?;
            std::fs::write(output.join(format!("{name}.{mode}.stderr")), &result.stderr)?;
            assert!(
                result.status.success(),
                "{name}/{mode}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        let (report, facts) = semantic::analyze(&text)?;
        if !canonical {
            assert!(
                report
                    .resolution
                    .library_types
                    .get(LibraryType::AtomicU32)
                    .is_none()
            );
            assert!(
                report
                    .resolution
                    .library_types
                    .get(LibraryType::Ordering)
                    .is_none()
            );
            assert_eq!(facts.intrinsics.iter().count(), 0, "{name}");
        }
    }
    Ok(())
}
