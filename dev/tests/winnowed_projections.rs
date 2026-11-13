//! A projection's symbolic result discards each compatible impl whose trait
//! condition has no candidate, as rustc's winnowing does. Core's
//! `(&[]).into_iter()` in `slice/iter/macros.rs`'s `Default` needs it: the
//! blanket `impl<I: Iterator> IntoIterator for I` matches `&[?E; 0]`'s header.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/blanket-candidate-winnowing";
const SUFFIX: &str = "impl<T> Default for Iter<'_, T> {
    fn default() -> Self {
        (& []).into_iter()
    }
}
";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "winnowed_projections")?;
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
            "minicore@winnowed_projections",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "winnowed_projections",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

/// Every accepted case types and lowers. Each rejection reports its message at
/// the last occurrence of its site.
#[test]
fn blanket_candidates_are_winnowed_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let mut columns = line.split('\t');
        let path = columns.next().ok_or("winnowing case path")?;
        let expected = columns.next().ok_or("winnowing case outcome")?;
        let (types, stderr) = typed(&root.join(path))?;
        let source = crate::resolve::fixture(path)?;
        let matches = if expected == "0" {
            let lowered = program(&source).map(|_| ());
            if let Err(error) = &lowered {
                failures.push(format!("{path}: lowering {error}"));
            }
            types && lowered.is_ok()
        } else {
            let site = columns.next().ok_or("winnowing rejection site")?;
            let offset = source.rfind(site).ok_or("winnowing site text")?;
            !types && stderr.starts_with(&format!("ERROR: file 0: {offset}: {expected}\n"))
        };
        if !matches {
            failures.push(format!("{path}: types={types} {stderr}"));
        }
        count += 1;
    }
    assert_eq!(count, 11);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `functions` functions, each returning `(&[]).into_iter()` as an `Iter`.
fn width(prefix: &str, functions: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..functions {
        writeln!(
            text,
            "pub fn empty{index}<'a, T>() -> Iter<'a, T> {{ (&[]).into_iter() }}"
        )?;
    }
    Ok(text)
}

/// Typing work stays linear in the winnowed projections, and every width lowers.
#[test]
fn winnowed_projection_work_tracks_uses() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/core-form.rs"))?;
    let prefix = source.strip_suffix(SUFFIX).ok_or("winnowing suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for functions in [8, 32, 128] {
        let text = width(prefix, functions)?;
        let report = crate::resolve::source_provider(&text, "winnowed_projections")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("winnowed projections functions={functions} work={work:?}");
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
