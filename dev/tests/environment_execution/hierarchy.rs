//! Complete generic sources precede requested receiver wrappers and ordinary cleanup.
use super::native;
use std::path::PathBuf;
const CASES: &[(&str, bool)] = &[
    ("owned-once", true),
    ("owned-mutable", true),
    ("mutable-once", true),
    ("reference-once", true),
    ("reference-mutable", true),
    ("mutable-reference-once", true),
    ("external-result", true),
    ("reference-result", true),
    ("late-argument", true),
    ("const-array", true),
    ("unused-arguments", true),
    ("projected-output", true),
    ("projected-reference", true),
    ("drop-once", true),
    ("nested-callable", true),
    ("unused-binder", true),
    ("move-twice", false),
    ("shared-mut-reject", false),
    ("borrow-conflict", false),
    ("owned-result-escape", false),
    ("false-output", false),
    ("quantified-input", true),
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/generic-receivers/{name}.rs"))
}
#[test]
fn generic_receiver_outcomes_preserve_declared_boundaries() -> Result<(), Box<dyn std::error::Error>>
{
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/generic-receivers/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&output)?;
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let result = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(fixture(name))
            .output()?;
        std::fs::write(output.join(format!("{name}.stderr")), &result.stderr)?;
        if result.status.code() != Some(i32::from(!accepted)) {
            failures.push(format!(
                "{name}: {:?}: {}",
                result.status,
                String::from_utf8_lossy(&result.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn generic_receiver_wrappers_execute_and_drop() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        if accepted {
            let source = std::fs::read_to_string(fixture(name))?;
            native::check_cases(
                "parallel/pa24/generic-receivers/native",
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

#[test]
fn generic_adapter_cache_tracks_complete_instances() -> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write as _;
    for distinct in [false, true] {
        let mut observations = Vec::new();
        for width in [8, 32, 128] {
            let mut source = String::from(
                "fn once<F:FnOnce()->u8>(f:F)->u8{f()}fn take<const N:usize>(x:u8)->u8{let f=move||x;once(f)}fn main(){",
            );
            for index in 0..width {
                let argument = if distinct { index } else { 0 };
                write!(source, "let _=take::<{argument}>(7);")?;
            }
            source.push('}');
            let (mut report, mut facts) = super::semantic::analyze(&source)?;
            let entry = rustgm::sema::entry::select(&mut report, &facts)?;
            let output = rustgm::lower::instances::build(&report, &mut facts, entry)?;
            let adapters = output
                .symbols
                .iter()
                .filter(|s| s.name.starts_with("rg.closure.adapter.") && s.body.is_some())
                .count();
            assert_eq!(adapters, if distinct { width } else { 1 });
            let work = facts.body_work.canonical_steps;
            assert!(work < 16384 * (width + 1), "{distinct}/{width}: {work}");
            eprintln!(
                "generic adapters distinct={distinct}/{width}: steps={work}, bodies={adapters}"
            );
            observations.push(work);
        }
        assert!(
            observations[2] - observations[1] <= 5 * (observations[1] - observations[0]),
            "{distinct}: {observations:?}"
        );
    }
    Ok(())
}
