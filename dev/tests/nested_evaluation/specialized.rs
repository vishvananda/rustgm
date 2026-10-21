//! Anonymous specializations retain complete enclosing identities and independent proofs.
use super::native;
const CASES: &[(&str, bool)] = &[
    ("distinct-recursion", true),
    ("cycle", false),
    ("lifetime-owner", true),
    ("lifetime-closure", true),
    ("associated", true),
    ("generic-type-check", true),
    ("known-zero", true),
    ("zero-value", true),
    ("const-arguments", true),
    ("early-late", true),
    ("nested", true),
    ("erased-type", true),
    ("array-result", true),
    ("generic-closure", true),
    ("captured-context", true),
    ("captured-erased", true),
    ("captured-late", true),
    ("zero-repeat", false),
    ("zero-erased", false),
    ("nonzero-erased", false),
    ("runtime-zero", true),
    ("zero-eligibility", false),
    ("const-loop", true),
    ("non-const", false),
    ("unused-nonconst", false),
    ("division", false),
    ("local-capture", false),
];
fn fixture(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa25/tests/spec/inline-specialized/{name}.rs")),
    )
}
#[test]
fn independently_qualified_specialized_inline_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        qualify(name, &fixture(name)?, accepted)?;
    }
    Ok(())
}
#[test]
fn complete_inline_instances_reach_checked_execution() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        let text = format!("{}fn main(){{let _=run();}}", fixture(name)?);
        let result = checked(&text);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
#[test]
#[ignore = "requires qualified CPPGM native backend"]
fn specialized_inline_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        if accepted {
            let text = format!("{}fn main(){{check(run());}}", fixture(name)?);
            native::check_cases(
                "parallel/pa25-inline-specialized/native",
                &[(name, &text, false)],
            )?;
        }
    }
    Ok(())
}

fn checked(text: &str) -> Result<(), String> {
    let (
        mut report,
        rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = super::analyze(text)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)
        .map_err(|e| e.to_string())?;
    Ok(())
}
fn qualify(name: &str, text: &str, accepted: bool) -> Result<(), Box<dyn std::error::Error>> {
    use std::{path::Path, process::Command};
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa25-inline-specialized/objects/{}/{name}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join("case.rs");
    let object = folder.join("case.o");
    std::fs::write(&input, text)?;
    let output = Command::new("rustc")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--crate-type=lib",
            "--crate-name=probe",
            "--emit=obj",
            "-o",
        ])
        .arg(&object)
        .arg(&input)
        .output()?;
    std::fs::write(folder.join("stderr"), &output.stderr)?;
    assert_eq!(
        output.status.code(),
        Some(i32::from(!accepted)),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    if accepted {
        std::fs::remove_file(object)?;
    }
    Ok(())
}

#[test]
fn exact_context_caches_reuse_original_cfgs() -> Result<(), Box<dyn std::error::Error>> {
    for count in [16usize, 64, 256] {
        use std::fmt::Write;
        let mut text = String::from("const fn f<const N:usize>()->usize{const{N+1}}fn main(){");
        for n in 0..count {
            write!(text, "let _=f::<{n}>();let _=f::<{n}>();")?;
        }
        text.push('}');
        let (
            mut report,
            rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            },
        ) = super::analyze(&text)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        let registry = facts
            .inline_constants
            .as_deref()
            .ok_or("missing inline registry")?;
        assert_eq!(registry.indexed, 1);
        assert_eq!(registry.instance_publications, count);
        assert_eq!(facts.constant_work.anonymous_templates, 1);
        assert_eq!(facts.constant_work.anonymous_instances, count);
        assert_eq!(facts.body_work.cfg_inline_expressions.get(), 1);
        assert!(registry.instance_lookups.get() < count * 32);
        assert!(registry.retained_bytes < count * 512 + 2048);
    }
    Ok(())
}

#[test]
fn specialized_constant_inventory_is_deterministic_and_keeps_templates_deferred()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa25-inline-specialized/inspection/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join("instances.rs");
    std::fs::write(&input, fixture("const-arguments")?)?;
    let mut observations = Vec::new();
    for index in 0..2 {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-constants")
            .arg(&input)
            .output()?;
        std::fs::write(folder.join(format!("{index}.stdout")), &output.stdout)?;
        std::fs::write(folder.join(format!("{index}.stderr")), &output.stderr)?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        observations.push(output.stdout);
    }
    assert_eq!(observations[0], observations[1]);
    let view = std::str::from_utf8(&observations[0])?;
    let roots = view
        .lines()
        .filter(|line| line.starts_with("root inline-instance "))
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 2);
    assert_ne!(roots[0], roots[1]);
    assert!(view.lines().any(|line| line.starts_with("declaration ")
        && line.contains("kind=inline")
        && line.contains("status=deferred-template")));
    Ok(())
}
