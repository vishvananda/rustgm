//! A derived tuple field index keeps its field's hygiene, so its privacy is
//! checked from the item's own module, as rustc's derive spans the index with
//! the field (`Ident::from_str_and_span(i, field.span)`). Core's private tuple
//! fields under derived `Debug` and `PartialEq` depend on it.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/derive-field-privacy";
const PRIVATE: &str = "aggregate field is private";
const SUFFIX: &str = "pub mod inner {\n    use crate::{derive, PartialEq};\n    #[derive(PartialEq)]\n    pub struct Token(u8);\n    pub fn same(a: &Token, b: &Token) -> bool { *a == *b }\n}\n";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "tuple_privacy")?;
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

/// The CLI's typing status and its stderr.
fn typed(path: &std::path::Path) -> Result<(bool, String)> {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-types",
            "--edition=2024",
            "--cap-lints=allow",
            "--source-core",
            "minicore@tuple_privacy",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "tuple_privacy",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

/// Every accepted case types and lowers; both outside accesses stay private.
#[test]
fn derived_tuple_indices_check_privacy_from_their_item_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("tuple privacy columns")?;
        let (types, stderr) = typed(&root.join(path))?;
        let matches = match expected {
            "0" => types && program(&crate::resolve::fixture(path)?).is_ok(),
            "1" => !types && stderr.contains(PRIVATE),
            _ => return Err("invalid tuple privacy outcome".into()),
        };
        if !matches {
            failures.push(format!("{path}: types={types} {stderr}"));
        }
        count += 1;
    }
    assert_eq!(count, 17);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `structs` derived tuple structs in one private module.
fn width(prefix: &str, structs: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    text.push_str("pub mod inner {\n    use crate::{derive, PartialEq};\n");
    for index in 0..structs {
        writeln!(
            text,
            "    #[derive(PartialEq)] pub struct T{index}(u8, bool);\n    pub fn same{index}(a: &T{index}, b: &T{index}) -> bool {{ *a == *b }}"
        )?;
    }
    text.push_str("}\n");
    Ok(text)
}

/// Typing work stays linear in the derived structs, and every width lowers.
#[test]
fn derived_tuple_index_work_tracks_structs() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/tuple-submodule.rs"))?;
    let prefix = source.strip_suffix(SUFFIX).ok_or("tuple privacy suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for structs in [8, 32, 128] {
        let text = width(prefix, structs)?;
        let report = crate::resolve::source_provider(&text, "tuple_privacy")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("tuple index privacy structs={structs} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 4 * old + 512,
                    "structs={structs}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
        program(&text)?;
    }
    Ok(())
}
