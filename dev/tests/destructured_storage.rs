//! Exact selected pattern types prepare storage without granting move or lifetime permission.
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/generic-pattern-storage";
const ACCEPTED: [&str; 6] = [
    "tuple",
    "record",
    "ergonomic",
    "nested",
    "parameter",
    "match",
];
fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(
        &crate::resolve::fixture(&format!("{PACKET}/{name}.rs"))?,
        "generic_pattern_storage",
        &config,
    )?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn selected_generic_pattern_fields_keep_lifetime_and_move_checks() -> Result {
    for name in ACCEPTED {
        program(name)?;
    }
    for (name, expected) in [
        (
            "unpaid-lifetime",
            "required lifetime does not outlive its target",
        ),
        (
            "move-from-reference",
            "cannot move a non-Copy value out of a reference",
        ),
    ] {
        let error = program(name)
            .err()
            .ok_or("invalid generic pattern accepted")?;
        assert!(error.to_string().contains(expected), "{name}: {error}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn generic_reference_patterns_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in ACCEPTED {
        crate::observed_program::execute(
            &root
                .join("obj/parallel/generic-pattern-storage/native")
                .join(name),
            &program(name)?,
            &root.join(PACKET).join("observe.c"),
            b"41 59\n",
        )?;
    }
    Ok(())
}

/// The selected pattern nodes are linear in source size. Preparing their
/// distinct generic field instances must keep that bound rather than replaying
/// the growing prefix of previously published bodies.
#[test]
fn pattern_field_preparation_scales_with_selected_source() -> Result {
    use std::fmt::Write as _;
    let fixture = crate::resolve::fixture(&format!("{PACKET}/tuple.rs"))?;
    let foundation = fixture
        .split_once("pub fn consume")
        .ok_or("tuple fixture foundation")?
        .0;
    let mut observations = Vec::new();
    for count in [8, 32, 128] {
        let mut source = foundation.to_owned();
        for index in 0..count {
            writeln!(
                source,
                "pub fn consume_{index}<T>(value:&T,tag:u8)->u8{{\
                 let mut guard=Guard(value,tag);let reference=&mut guard;\
                 let &mut Guard(_,n)=reference;n}}"
            )?;
        }
        qualify_scaling(count, &source)?;
        let report = crate::resolve::source_provider(&source, "pattern_storage_scaling")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        observations.push([
            facts.aggregates.instantiation_steps(),
            facts.body_work.canonical_steps,
        ]);
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/generic-pattern-storage/scaling")
        .join(std::process::id().to_string());
    std::fs::write(
        folder.join("observations.json"),
        format!(
            "{{\"counts\":[8,32,128],\"metrics\":[\"field_instantiation_steps\",\"canonical_steps\"],\"observations\":{observations:?}}}\n"
        ),
    )?;
    for dimension in 0..2 {
        let small = observations[0][dimension];
        let middle = observations[1][dimension];
        let large = observations[2][dimension];
        assert!(small < middle && middle < large, "{observations:?}");
        // Source grows by 24 then 96 functions. A linear walk has 4 times the
        // incremental work; the 2 times slack rejects quadratic prefix replay.
        assert!(large - middle <= (middle - small) * 8, "{observations:?}");
    }
    Ok(())
}

fn qualify_scaling(count: usize, source: &str) -> Result {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/generic-pattern-storage/scaling")
        .join(std::process::id().to_string())
        .join(count.to_string());
    std::fs::create_dir_all(&folder)?;
    let input = folder.join("input.rs");
    let object = folder.join("input.o");
    std::fs::write(&input, source)?;
    let output = std::process::Command::new("rustc")
        .env("RUSTC_BOOTSTRAP", "1")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--crate-type=lib",
            "--emit=obj",
            "--cap-lints=allow",
            "-Cpanic=abort",
            "-o",
        ])
        .arg(&object)
        .arg(&input)
        .output()?;
    std::fs::write(folder.join("rustc.stderr"), &output.stderr)?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::remove_file(object)?;
    Ok(())
}
