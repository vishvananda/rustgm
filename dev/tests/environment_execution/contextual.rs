//! Generic callable context preserves quantified inputs and ordinary lifetime proofs.
use super::{native, semantic};
use std::{path::PathBuf, process::Command};

const CASES: &[(&str, bool)] = &[
    ("captured-identity", true),
    ("empty-identity", true),
    ("explicit-identity", true),
    ("nested-result", true),
    ("mutable-call", true),
    ("owned-call", true),
    ("separate-inputs", true),
    ("crossed-output", false),
    ("local-escape", false),
    ("static-escape", false),
    ("captured-escape", false),
    ("unused-binder", true),
    ("generic-input", true),
    ("nested-callable", true),
    ("fn-mut-denied", false),
    ("uncontextual-identity", false),
    ("unused-identity", true),
    ("unused-captured-identity", true),
];

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/generic-context/{name}.rs"))
}

#[test]
fn generic_context_matches_independent_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/contextual-capture/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(name, accepted) in CASES {
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
        let expected = i32::from(!accepted);
        assert_eq!(
            independent.status.code(),
            Some(expected),
            "{name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(path)
            .output()?;
        std::fs::write(folder.join(format!("{name}.stdout")), &output.stdout)?;
        std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if !accepted {
            let diagnostic = if name == "fn-mut-denied" {
                "trait obligation requires an unambiguous matching assumption or implementation"
            } else {
                "required lifetime does not outlive its target"
            };
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
fn generic_context_executes_quantified_results() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        if accepted {
            let source = std::fs::read_to_string(fixture(name))?;
            let body = source
                .strip_prefix(native::CHECK)
                .ok_or("missing fixture check")?;
            native::check_cases(
                "parallel/pa24/contextual-capture/native",
                &[(name, body, false)],
            )?;
        }
    }
    Ok(())
}

#[test]
fn independent_contexts_keep_binders_and_bounded_work() -> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut source = String::from("fn infer<F:for<'a>Fn(&'a u8)->&'a u8>(f:F)->F{f}fn main(){");
        for index in 0..width {
            write!(source, "let _f{index}=infer(|x|x);")?;
        }
        source.push('}');
        let (_, facts) = semantic::analyze(&source)?;
        assert_eq!(facts.closures.all().len(), width);
        for body in facts.closures.all() {
            assert!(matches!(
                facts.types.kind(body.signature),
                Some(rustgm::types::Kind::Function {
                    bound_regions: 1,
                    ..
                })
            ));
        }
        assert!(facts.body_work.canonical_steps < 4096 * (width + 1));
        observations.push(facts.body_work.canonical_steps);
    }
    assert!(observations[2] - observations[1] <= 5 * (observations[1] - observations[0]));
    Ok(())
}
