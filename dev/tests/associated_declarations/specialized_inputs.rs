//! Specialization orders candidates whose inferred trait inputs agree, as in core's `Zip`.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/specialized-trait-inputs";
const REJECTED: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";

fn cases() -> Result<Vec<(String, bool)>> {
    let mut cases = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line
            .split_once('\t')
            .ok_or("specialized input outcome columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid specialized input outcome".into()),
        };
        cases.push((path.to_owned(), accepted));
    }
    assert_eq!(cases.len(), 9);
    Ok(cases)
}

fn run(mode: &str, path: &str) -> Result<std::process::Output> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    Ok(std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            mode,
            "--edition=2024",
            "--cap-lints=allow",
            "--source-core",
            "minicore@specialized_inputs",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "specialized_inputs",
        ])
        .arg(root.join(path))
        .output()?)
}

/// `call-concrete` also lowers both calls from a root `main`. Only the
/// specializing bodies (`1`) may be lowered; the default bodies return `0`,
/// which only the entry point also returns.
#[test]
fn specializing_candidates_with_agreeing_inferred_inputs_select_like_rustc() -> Result {
    let mut failures = Vec::new();
    for (path, accepted) in cases()? {
        let lowered = path.ends_with("/call-concrete.rs");
        for mode in ["--emit-types", "--emit-lowir"] {
            if mode == "--emit-lowir" && !lowered {
                continue;
            }
            let output = run(mode, &path)?;
            let stderr = String::from_utf8_lossy(&output.stderr);
            let expected = if accepted { Some(0) } else { Some(1) };
            if output.status.code() != expected || !(accepted || stderr.contains(REJECTED)) {
                failures.push(format!(
                    "{path} {mode}: {:?} {stderr}",
                    output.status.code()
                ));
            }
            if lowered && mode == "--emit-lowir" {
                let program = String::from_utf8_lossy(&output.stdout);
                let defaults = program.matches("return i64 0").count();
                let specialized = program.matches("return i64 1").count();
                if (defaults, specialized) != (1, 2) {
                    failures.push(format!("{path}: lowered {defaults} default/entry and {specialized} specialized returns"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// Unrelated specialized pairs for other self types.
fn width(prefix: &str, calls: usize, noise: bool) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..calls {
        writeln!(text, "fn c{index}(z:&Z<S,S>)->usize{{ZImpl::size(z)}}")?;
    }
    for index in 0..if noise { 64 } else { 0 } {
        writeln!(
            text,
            "pub struct N{index}<A,B>(A,B);trait P{index}<A,B>{{fn p(&self)->usize;}}impl<A:It,B:It> P{index}<A,B> for N{index}<A,B>{{default fn p(&self)->usize{{0}}}}impl<A:Fast+It,B:Fast+It> P{index}<A,B> for N{index}<A,B>{{fn p(&self)->usize{{1}}}}"
        )?;
    }
    Ok(text)
}

#[test]
fn agreeing_input_winnowing_tracks_calls_without_scanning_unrelated_pairs() -> Result {
    let prefix = crate::resolve::fixture(&format!("{PACKET}/caller-concrete.rs"))?;
    let mut previous = None;
    let mut unrelated = None;
    for calls in [8, 32, 128] {
        let mut ordinary = [0; 3];
        for noise in [false, true] {
            let report = crate::resolve::source_provider(
                &width(&prefix, calls, noise)?,
                "specialized_inputs",
            )?;
            let facts = rustgm::pipeline::analyze_source(&report)?.facts;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
            ];
            println!("specialized inputs calls={calls} noise={noise} work={work:?}");
            if noise {
                // The unrelated pairs pay their own declarations once; these
                // calls never select or order them.
                let added: Vec<_> = std::iter::zip(ordinary, work)
                    .map(|(old, actual)| actual.checked_sub(old))
                    .collect();
                assert_eq!(*unrelated.get_or_insert_with(|| added.clone()), added);
            } else {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(
                            actual <= 4 * old + 512,
                            "calls={calls}: {before:?}->{work:?}"
                        );
                    }
                }
                previous = Some(work);
                ordinary = work;
            }
        }
    }
    Ok(())
}
