//! A source `impl Copy` lets a closure copy its capture and stay callable by reference.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/source-copy-captures";
const REJECTED: &str = "quantified body requirement needs a matching assumption or implementation";

fn cases() -> Result<Vec<(String, bool)>> {
    let mut cases = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line
            .split_once('\t')
            .ok_or("copy capture outcome columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid copy capture outcome".into()),
        };
        cases.push((path.to_owned(), accepted));
    }
    assert_eq!(cases.len(), 12);
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
            "minicore@copy_captures",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "copy_captures",
        ])
        .arg(root.join(path))
        .output()?)
}

/// Core's `NonZero` and conditional user `impl Copy` captures, through the CLI.
/// The `call-*` cases also lower their copying bodies from a root `main`.
#[test]
fn source_copy_implementations_keep_copying_closures_callable_by_reference() -> Result {
    let mut failures = Vec::new();
    for (path, accepted) in cases()? {
        let lowered = path.contains("/call-");
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
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// Unrelated source Copy types, each with its own `Clone` body.
fn width(
    prefix: &str,
    captures: usize,
    noise: bool,
) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..captures {
        writeln!(
            text,
            "fn c{index}(step:UsizeInner)->impl Fn(usize)->usize{{move |n|{{let _s=step;n}}}}"
        )?;
    }
    for index in 0..if noise { 128 } else { 0 } {
        writeln!(
            text,
            "pub struct M{index}(usize);impl Clone for M{index}{{fn clone(&self)->Self{{*self}}}}impl Copy for M{index}{{}}"
        )?;
    }
    Ok(text)
}

#[test]
fn implemented_copy_probe_tracks_captures_without_scanning_unrelated_copy_types() -> Result {
    let prefix = crate::resolve::fixture(&format!("{PACKET}/inner.rs"))?;
    let mut previous = None;
    let mut unrelated = None;
    for captures in [8, 32, 128] {
        let mut ordinary = [0; 3];
        for noise in [false, true] {
            let report = crate::resolve::source_provider(
                &width(&prefix, captures, noise)?,
                "copy_captures",
            )?;
            let facts = rustgm::pipeline::analyze_source(&report)?.facts;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
            ];
            println!("copy captures={captures} noise={noise} work={work:?}");
            if noise {
                // The unrelated types check their own `Clone` bodies once; the
                // capture probes never select or refute them.
                let added: Vec<_> = std::iter::zip(ordinary, work)
                    .map(|(old, actual)| actual.checked_sub(old))
                    .collect();
                assert_eq!(*unrelated.get_or_insert_with(|| added.clone()), added);
            } else {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(
                            actual <= 4 * old + 512,
                            "captures={captures}: {before:?}->{work:?}"
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
