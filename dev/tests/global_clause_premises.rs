//! A normalization whose trait premise is concrete selects it from an
//! implementation before a caller's global where clause, as rustc discards
//! such a clause when an impl applies. Core's portable-simd `Div` for
//! `Simd<i8, N>` (`ops.rs`) declares `where i8: SimdElement`, and its body's
//! `select` needs `<i8 as SimdElement>::Mask`'s normal form.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/global-clause-premises";
const TAG: &str = "global_clause_premises";

fn lowered(source: &str) -> Result {
    let report = crate::resolve::source_provider(source, TAG)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok(())
}

/// The CLI's typing status and its stderr for one packet file.
fn cli_types(path: &std::path::Path) -> Result<(bool, String)> {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args(["--emit-types", "--edition=2024", "--cap-lints=allow"])
        .args(["--source-core", &format!("minicore@{TAG}")])
        .args(["--crate-name", "minicore", "--crate-disambiguator", TAG])
        .arg(path)
        .output()?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok((output.status.success(), stderr))
}

/// Every accepted case types and lowers, and each rejection reports its
/// message at the last occurrence of its site.
#[test]
fn global_clause_premises_select_implementations_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut failures = Vec::new();
    let cases = crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?;
    for line in cases.lines() {
        let columns: Vec<&str> = line.split('\t').collect();
        let [path, outcome, rest @ ..] = columns.as_slice() else {
            return Err(format!("malformed case row {line}").into());
        };
        let (types, stderr) = cli_types(&root.join(path))?;
        let source = crate::resolve::fixture(path)?;
        if *outcome == "0" {
            if let Err(error) = lowered(&source) {
                failures.push(format!("{path}: lowering {error}"));
            }
            if !types {
                failures.push(format!("{path}: {stderr}"));
            }
            continue;
        }
        let site = rest.first().ok_or("rejection site")?;
        let offset = source.rfind(site).ok_or("rejection site text")?;
        if types || !stderr.starts_with(&format!("ERROR: file 0: {offset}: {outcome}\n")) {
            failures.push(format!("{path}: types={types} {stderr}"));
        }
    }
    assert_eq!(cases.lines().count(), 25);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `case`'s prefix followed by `functions` copies of its `select` function,
/// `f0`, `f1` and so on.
fn copies(case: &str, functions: usize) -> Result<String> {
    let source = crate::resolve::fixture(&format!("{PACKET}/{case}.rs"))?;
    let start = source.find("pub fn f<").ok_or("select function")?;
    let function = source[start..].trim_end();
    let mut text = source[..start].to_owned();
    for index in 0..functions {
        writeln!(
            text,
            "{}",
            function.replacen("pub fn f<", &format!("pub fn f{index}<"), 1)
        )?;
    }
    Ok(text)
}

/// Inference, canonical and trait selection steps of one typed library.
fn work(text: &str) -> Result<[usize; 3]> {
    let report = crate::resolve::source_provider(text, TAG)?;
    let facts = rustgm::pipeline::analyze_source(&report)?.facts;
    Ok([
        facts.body_work.inference_steps,
        facts.body_work.canonical_steps,
        facts.body_work.trait_selections,
    ])
}

/// An implementation-first premise is not cached, so each body repeats its
/// search. Work stays linear in the functions that declare the clause and
/// within twice the clause-free `select-splat-cast` form. The widths stay
/// below the crate-wide source method selection budget, which 128 copies of
/// either form exhaust on the base as well.
#[test]
fn global_clause_premise_work_tracks_functions() -> Result {
    let mut previous: Option<[usize; 3]> = None;
    for functions in [4, 16, 64] {
        let text = copies("global-where-select", functions)?;
        let clause = work(&text)?;
        let control = work(&copies("select-splat-cast", functions)?)?;
        println!(
            "global clause premises functions={functions} work={clause:?} control={control:?}"
        );
        for (actual, free) in std::iter::zip(clause, control) {
            assert!(
                actual <= 2 * free + 512,
                "functions={functions}: {clause:?} against {control:?}"
            );
        }
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, clause) {
                assert!(
                    actual <= 4 * old + 512,
                    "functions={functions}: {before:?}->{clause:?}"
                );
            }
        }
        previous = Some(clause);
        lowered(&text)?;
    }
    Ok(())
}
