//! A receiver step whose trait requirement meets a negative impl does not
//! apply, as for `t.clone()` on `&mut T` beside core's
//! `impl !Clone for &mut T`; autoderef continues to `T: Clone`. Ordinary
//! bound obligations keep the conditional-negative boundary.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/negative-receivers";
const CONDITIONAL: &str = "conditional negative implementation requires scoped debt payment";
const REJECTED: [(&str, &str); 2] = [
    ("missing.rs", "no applicable method for this receiver"),
    ("bound-obligation.rs", CONDITIONAL),
];
const CALL: &str = "pub fn f<T: Clone>(t: &mut T) -> T { t.clone() }\n";

fn cases() -> Result<Vec<(String, bool)>> {
    let mut cases = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line
            .split_once('\t')
            .ok_or("negative receiver outcome columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid negative receiver outcome".into()),
        };
        cases.push((path.to_owned(), accepted));
    }
    assert_eq!(cases.len(), 10);
    Ok(cases)
}

#[test]
fn negative_requirements_leave_receiver_steps_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut failures = Vec::new();
    for (path, accepted) in cases()? {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-types",
                "--edition=2024",
                "--cap-lints=allow",
                "--source-core",
                "minicore@negative_receivers",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "negative_receivers",
            ])
            .arg(root.join(&path))
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = REJECTED
            .iter()
            .find(|(name, _)| path.ends_with(&format!("/{name}")))
            .map(|(_, reason)| *reason);
        let expected = if accepted { Some(0) } else { Some(1) };
        if output.status.code() != expected
            || accepted != reason.is_none()
            || reason.is_some_and(|reason| !stderr.contains(reason))
        {
            failures.push(format!("{path}: {:?} {stderr}", output.status.code()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `calls` functions cloning through a `&mut T` receiver beside core's
/// negative impl, then unrelated nominals with their own `Clone` impls.
fn width(prefix: &str, calls: usize, noise: bool) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..calls {
        writeln!(
            text,
            "pub fn c{index}<T: Clone>(t: &mut T) -> T {{ t.clone() }}"
        )?;
    }
    for index in 0..if noise { 64 } else { 0 } {
        writeln!(
            text,
            "pub struct N{index}(u8);impl Clone for N{index}{{fn clone(&self)->Self{{N{index}(self.0)}}}}"
        )?;
    }
    Ok(text)
}

#[test]
fn negative_receiver_steps_track_calls_without_scanning_unrelated_impls() -> Result {
    let direct = crate::resolve::fixture(&format!("{PACKET}/direct.rs"))?;
    let prefix = direct.strip_suffix(CALL).ok_or("direct receiver call")?;
    let mut previous = None;
    let mut unrelated = None;
    for calls in [8, 32, 128] {
        let mut ordinary = [0; 3];
        for noise in [false, true] {
            let report = crate::resolve::source_provider(
                &width(prefix, calls, noise)?,
                "negative_receivers",
            )?;
            let facts = rustgm::pipeline::analyze_source(&report)?.facts;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
            ];
            println!("negative receivers calls={calls} noise={noise} work={work:?}");
            if noise {
                // The unrelated impls check their own bodies once; the
                // receiver steps never select them.
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
