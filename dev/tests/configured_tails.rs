//! A block whose written tail is configured out takes its last enabled
//! statement written without a semicolon as its tail, as rustc does for core's
//! `fmt/num.rs` `Display` bodies. Such a statement elsewhere must have type
//! `()`, including an interpolated statement followed by `;`.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/configured-tails";
const MISMATCH: &str = "incompatible types";
const SUFFIX: &str = "pub fn f() -> u8 {\n    #[cfg(not(feature = \"x\"))]\n    {\n        1\n    }\n    #[cfg(feature = \"x\")]\n    {\n        2\n    }\n}\n";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "cfg_tail")?;
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
            "minicore@cfg_tail",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "cfg_tail",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

/// Every accepted case types and lowers; every rejection is a type mismatch.
#[test]
fn configured_out_tails_and_unterminated_statements_type_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("configured tail columns")?;
        let (types, stderr) = typed(&root.join(path))?;
        let matches = match expected {
            "0" => types && program(&crate::resolve::fixture(path)?).is_ok(),
            "1" => !types && stderr.contains(MISMATCH),
            _ => return Err("invalid configured tail outcome".into()),
        };
        if !matches {
            failures.push(format!("{path}: types={types} {stderr}"));
        }
        count += 1;
    }
    assert_eq!(count, 16);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `functions` functions, each with a configured-out tail after an enabled block.
fn width(prefix: &str, functions: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..functions {
        writeln!(
            text,
            "pub fn f{index}() -> u8 {{ #[cfg(not(feature = \"x\"))] {{ 1 }} #[cfg(feature = \"x\")] {{ 2 }} }}"
        )?;
    }
    Ok(text)
}

/// Typing work stays linear in the promoted tails, and every width lowers.
#[test]
fn configured_tail_work_tracks_functions() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/tail-cfg-out.rs"))?;
    let prefix = source
        .strip_suffix(SUFFIX)
        .ok_or("configured tail suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for functions in [8, 32, 128] {
        let text = width(prefix, functions)?;
        let report = crate::resolve::source_provider(&text, "cfg_tail")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("configured tails functions={functions} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 4 * old + 512,
                    "functions={functions}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
        program(&text)?;
    }
    Ok(())
}
