//! Captured conversions consume the selected pointee and retain ordinary borrow proof.
use super::{native, semantic};
use std::{path::PathBuf, process::Command};

const CASES: &[(&str, Option<&str>)] = &[
    ("shared-from-mut", None),
    ("mutable-reborrow", None),
    (
        "mutable-immutable-call",
        Some("cannot mutably borrow an immutable local"),
    ),
    ("shared-deep", None),
    ("shared-slice", None),
    ("mutable-slice", None),
    ("shared-slice-from-mut", None),
    ("field-disjoint", None),
    (
        "shared-conflict",
        Some("access conflicts with a live reference loan"),
    ),
    (
        "mutable-conflict",
        Some("access conflicts with a live reference loan"),
    ),
    (
        "shared-cannot-mutate",
        Some("cannot coerce a shared reference to a mutable reference"),
    ),
    ("move-reborrow", None),
    ("nested-shared", None),
    ("nested-mutable", None),
    ("borrowed-output", None),
    ("reference-slot-loan", None),
];

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../pa24/tests/spec/reference-conversions/{name}.rs"
    ))
}

#[test]
fn declared_reference_conversion_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/reference-captures/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(name, diagnostic) in CASES {
        let path = fixture(name);
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
            .arg(folder.join(format!("{name}.o")))
            .arg(&path)
            .output()?;
        std::fs::write(
            folder.join(format!("{name}.rustc.stderr")),
            &independent.stderr,
        )?;
        let expected = i32::from(diagnostic.is_some());
        assert_eq!(
            independent.status.code(),
            Some(expected),
            "{name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&path)
            .output()?;
        std::fs::write(folder.join(format!("{name}.stdout")), &output.stdout)?;
        std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if let Some(diagnostic) = diagnostic {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(diagnostic),
                "{name}"
            );
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn captured_reference_conversions_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, diagnostic) in CASES {
        if diagnostic.is_none() {
            let source = std::fs::read_to_string(fixture(name))?;
            let body = source
                .strip_prefix(native::CHECK.trim_end())
                .ok_or("missing fixture checks")?;
            native::check_cases(
                "parallel/pa24/reference-captures/native",
                &[(name, body, false)],
            )?;
        }
    }
    Ok(())
}

#[test]
fn conversion_cannot_skip_unselected_capture_dereferences() -> Result<(), Box<dyn std::error::Error>>
{
    let source = std::fs::read_to_string(fixture("shared-deep"))?;
    let (report, mut facts) = semantic::analyze(&source)?;
    let selected = facts
        .reference_adjustments
        .values_mut()
        .find(|reference| reference.conversion.dereferences == 2)
        .ok_or("missing deep conversion")?;
    selected.conversion.dereferences = 1;
    let error = rustgm::body::build_concrete(&report, &facts)
        .err()
        .ok_or("shortened selected conversion was accepted")?;
    assert!(
        error
            .to_string()
            .contains("capture prefix exceeds selected receiver dereferences")
    );
    Ok(())
}

#[test]
fn repeated_conversions_share_capture_path_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut source =
            String::from("fn read(x:&u8)->u8{*x}fn main(){let mut n=7u8;let r=&mut n;let f=||{");
        for _ in 0..width {
            source.push_str("let _=read(r);");
        }
        source.push_str("};f();}");
        let (_, facts) = semantic::analyze(&source)?;
        let body = facts.closures.all().first().ok_or("missing closure")?;
        let environment = facts
            .closures
            .environment(body.identity)
            .ok_or("missing capture")?;
        assert_eq!(environment.plan.captures().len(), 1);
        assert_eq!(environment.plan.path_count(), 2);
        assert_eq!(environment.adjusted_accesses.len(), width);
        assert!(facts.body_work.canonical_steps < 1024 * (width + 1));
        observations.push(facts.body_work.canonical_steps);
    }
    assert_eq!(
        observations[2] - observations[1],
        4 * (observations[1] - observations[0])
    );
    Ok(())
}
