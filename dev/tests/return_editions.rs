//! Return opaque lifetime captures follow the introducer's defining edition.
use rustgm::support::edition::Edition;
use std::path::Path;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CASES: &[(&str, bool, bool)] = &[
    ("implicit-hidden-reference", false, true),
    ("explicit-outlives-reference", true, true),
    ("explicit-precise-reference", true, true),
    ("excluded-hidden-reference", false, false),
    ("implicit-scalar-escape", true, false),
    ("precise-scalar-escape", true, true),
    ("inherent-scalar-escape", true, false),
    ("inherent-hidden-reference", false, true),
    ("trait-default-hidden-reference", true, true),
    ("trait-override-hidden-reference", true, true),
    ("where-clause-does-not-capture", false, true),
    ("alias-erased-bound-captures", true, true),
    ("associated-alias-bound-captures", true, true),
    ("quantified-local-does-not-capture", false, true),
    ("quantified-explicit-outer-capture", true, true),
    ("precise-type-const-capture", true, true),
];

fn source(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa34/tests/spec/opaque-editions")
            .join(format!("{name}.rs")),
    )?)
}

#[test]
fn source_capture_outcomes_preserve_written_alias_and_quantifier_lifetimes() -> Result {
    let mut failures = Vec::new();
    for &(name, old, new) in CASES {
        for (edition, accepted) in [(Edition::Rust2021, old), (Edition::Rust2024, new)] {
            match crate::edition_packages::lowir(&source(name)?, edition) {
                Ok(bytes) if accepted && !bytes.is_empty() => {}
                Ok(_) => failures.push(format!("{name}/{edition:?}: unexpected acceptance")),
                Err(error) if accepted => {
                    failures.push(format!("{name}/{edition:?}: {error}"));
                }
                Err(error) => {
                    let message = error.to_string();
                    if message.contains("unsupported")
                        || !["lifetime", "capture", "borrow", "outlive", "loan"]
                            .iter()
                            .any(|word| message.contains(word))
                    {
                        failures.push(format!("{name}/{edition:?}: unrelated rejection: {error}"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// Install donor origins only for transcribed macro-definition bytes. Captured
/// return tokens keep their invocation origin; this is not a metadata import.
fn macro_lowir(kind: &str, edition: Edition, definition: Edition) -> Result<Vec<u8>> {
    use rustgm::{
        expand::config::Config,
        resolve::unit::{FileId, Unit},
        support::source::SourceFile,
    };
    let provider = source(&format!("macro-provider-{kind}"))?.replace("#[macro_export]", "");
    let client = source(&format!("macro-client-{kind}-2021-2021"))?.replace("macros::", "");
    let text = format!("{provider}{client}");
    let mut report = crate::edition_packages::analyze(
        &text,
        edition,
        &mut crate::edition_packages::Files::default(),
    )?;
    let donor = Unit::parse(
        SourceFile::new(text.as_bytes().to_vec())?,
        &Config::in_edition(definition),
        &mut report.package.names,
    )
    .map_err(|error| format!("{error:?}"))?;
    let mut redirected = 0;
    for origin in &mut report.package.units[0].origins {
        if origin.original.file == FileId(0) && origin.original.span.end as usize <= provider.len()
        {
            origin.original.file = FileId(1);
            redirected += 1;
        }
    }
    assert!(redirected > 0);
    report.package.units.push(donor);
    report.package.file_paths.push("/crate/donor.rs".into());
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let program = rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    let mut bytes = Vec::new();
    rustgm::lowir::writer::write(&program, &mut bytes)?;
    Ok(bytes)
}

#[test]
fn transcribed_and_captured_return_tokens_keep_their_own_editions() -> Result {
    for kind in ["defined", "captured"] {
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            for definition in [Edition::Rust2021, Edition::Rust2024] {
                let selected = if kind == "defined" {
                    definition
                } else {
                    edition
                };
                let result = macro_lowir(kind, edition, definition);
                if selected == Edition::Rust2024 {
                    assert!(!result?.is_empty(), "{kind}/{edition:?}/{definition:?}");
                } else {
                    let error = result
                        .err()
                        .ok_or("2021 hidden lifetime must be rejected")?;
                    assert!(error.to_string().contains("lifetime"), "{error}");
                }
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires CPPGM backend; pinned-rustc source/native outcomes were frozen first"]
fn captured_results_execute_without_changing_the_public_opaque_contract() -> Result {
    use std::process::Command;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join("obj/parallel/pa34-return-native").join(format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder)?;
    let execute = |name: &str, bytes: Vec<u8>| -> Result {
        let input = folder.join(format!("{name}.lowir"));
        let binary = folder.join(format!("{name}.candidate"));
        std::fs::write(&input, bytes)?;
        crate::edition_packages::run(
            &folder,
            &format!("{name}-backend"),
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg(input)
                .arg("-o")
                .arg(&binary),
        )?;
        crate::edition_packages::run(&folder, name, &mut Command::new(binary))?;
        for stream in ["stdout", "stderr"] {
            assert!(std::fs::read(folder.join(format!("{name}.{stream}")))?.is_empty());
        }
        Ok(())
    };
    for &(name, old, new) in CASES {
        for (edition, accepted) in [(Edition::Rust2021, old), (Edition::Rust2024, new)] {
            if accepted {
                execute(
                    &format!("{name}-{edition:?}"),
                    crate::edition_packages::lowir(&source(name)?, edition)?,
                )?;
            }
        }
    }
    for kind in ["defined", "captured"] {
        for other in [Edition::Rust2021, Edition::Rust2024] {
            let (edition, definition) = if kind == "defined" {
                (other, Edition::Rust2024)
            } else {
                (Edition::Rust2024, other)
            };
            execute(
                &format!("{kind}-{edition:?}-{definition:?}"),
                macro_lowir(kind, edition, definition)?,
            )?;
        }
    }
    Ok(())
}
