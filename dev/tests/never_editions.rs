//! Never-coercion defaults use the compiling crate, including generated bodies.
use rustgm::support::edition::Edition;
use std::{path::Path, process::Command};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const EDITIONS: &[(Edition, &str)] = &[(Edition::Rust2021, "2021"), (Edition::Rust2024, "2024")];
const POSITIVE: &[&str] = &[
    "native-fallback",
    "native-closure",
    "native-macro",
    "explicit-unit",
    "numeric-constraint",
];

fn source(name: &str, suffix: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa34/tests/spec/never-editions")
            .join(format!("{name}-{suffix}.rs")),
    )?)
}

#[test]
fn source_fallback_and_opposing_bounds_follow_the_compilation_edition() -> Result {
    for &(edition, suffix) in EDITIONS {
        for name in POSITIVE {
            let bytes = crate::edition_packages::lowir(&source(name, suffix)?, edition)
                .map_err(|error| format!("{name}/{suffix}: {error}"))?;
            assert!(!bytes.is_empty());
        }
        for (name, accepted) in [
            ("unit-only", edition == Edition::Rust2021),
            ("never-only", edition == Edition::Rust2024),
            ("ordinary-unknown", false),
        ] {
            match crate::edition_packages::lowir(&source(name, suffix)?, edition) {
                Ok(bytes) => assert!(accepted && !bytes.is_empty(), "{name}/{suffix}"),
                Err(error) => {
                    assert!(!accepted, "{name}/{suffix}: {error}");
                    let message = error.to_string();
                    assert!(!message.contains("unsupported"), "{name}/{suffix}: {error}");
                    let words: &[&str] = if name == "ordinary-unknown" {
                        &["unknown", "unresolved", "inference", "annotations needed"]
                    } else {
                        &["trait", "bound", "implementation"]
                    };
                    assert!(words.iter().any(|word| message.contains(word)), "{error}");
                }
            }
        }
    }
    Ok(())
}

fn generated_from_other_edition(source: &str, edition: Edition) -> Result<Vec<u8>> {
    use rustgm::{
        expand::config::Config,
        resolve::unit::{FileId, Unit},
        support::source::SourceFile,
        syntax::{ast::MacroContext, item::ItemContext},
    };
    let mut report = crate::edition_packages::analyze(
        source,
        edition,
        &mut crate::edition_packages::Files::default(),
    )?;
    let other = match edition {
        Edition::Rust2021 => Edition::Rust2024,
        Edition::Rust2024 => Edition::Rust2021,
    };
    let definition = report
        .definitions
        .iter()
        .find_map(|definition| definition.node)
        .ok_or("missing source macro definition")?;
    assert_eq!(definition.file, FileId(0));
    let span = report.package.units[0]
        .configured
        .ast
        .node(definition.node)
        .span;
    let original = source
        .get(span.start as usize..span.end as usize)
        .ok_or("macro definition exceeds its source")?;
    // This unit is retained provenance, not another installed crate/module.
    // Parse the actual macro fragment through the ordinary include-template API.
    let donor = Unit::parse_include(
        SourceFile::new(original.as_bytes().to_vec())?,
        &Config::in_edition(other),
        MacroContext::Items(ItemContext::Module),
        &mut report.package.names,
    )
    .map_err(|error| format!("{error:?}"))?;
    let unit = &mut report.package.units[0];
    assert!(!unit.origins.is_empty());
    let mut redirected = 0;
    for origin in &mut unit.origins {
        if origin.original.file == FileId(0)
            && origin.original.span.start >= span.start
            && origin.original.span.end <= span.end
        {
            origin.original.file = FileId(1);
            origin.original.span.start -= span.start;
            origin.original.span.end -= span.start;
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
fn opposite_token_origins_reach_source_validation_and_lowir() -> Result {
    for &(edition, suffix) in EDITIONS {
        assert!(
            !generated_from_other_edition(&source("native-macro", suffix)?, edition)?.is_empty()
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires CPPGM native backend; independent pinned-rustc outcomes are frozen"]
fn inferred_trait_selection_executes_with_local_and_foreign_token_editions() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join("obj/parallel/pa34-never-native").join(format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(edition, suffix) in EDITIONS {
        let bound = if edition == Edition::Rust2021 {
            "unit-only"
        } else {
            "never-only"
        };
        for name in POSITIVE.iter().copied().chain([bound, "foreign-macro"]) {
            let bytes = if name == "foreign-macro" {
                generated_from_other_edition(&source("native-macro", suffix)?, edition)?
            } else {
                crate::edition_packages::lowir(&source(name, suffix)?, edition)?
            };
            let tag = format!("{name}-{suffix}");
            let input = folder.join(format!("{tag}.lowir"));
            let binary = folder.join(format!("{tag}.candidate"));
            std::fs::write(&input, bytes)?;
            crate::edition_packages::run(
                &folder,
                &format!("{tag}-backend"),
                Command::new("python3")
                    .arg(root.join("scripts/backend.py"))
                    .arg(input)
                    .arg("-o")
                    .arg(&binary),
            )?;
            crate::edition_packages::run(&folder, &tag, &mut Command::new(binary))?;
            for stream in ["stdout", "stderr"] {
                assert!(std::fs::read(folder.join(format!("{tag}.{stream}")))?.is_empty());
            }
        }
    }
    Ok(())
}
