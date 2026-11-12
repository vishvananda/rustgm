//! A method call on `Pin<&mut SyncView<T>>` reaches `SyncView`'s inherent
//! `self: Pin<&mut Self>` method through the genuine LegacyReceiver/Deref
//! chain and matches it against the outer receiver, as core's `poll` needs.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa43/tests/spec/pinned-method-calls";
const REJECTED: [(&str, &str); 3] = [
    ("missing.rs", "no inherent method with this name"),
    (
        "unpinned.rs",
        "no applicable inherent method for this receiver",
    ),
    (
        "two-owners.rs",
        "multiple applicable methods for this receiver",
    ),
];
const CALL: &str =
    "pub fn project<T: ?Sized>(p: Pin<&mut SyncView<T>>) -> Pin<&mut T> { p.as_pin_mut() }\n";

#[test]
fn pinned_inherent_receivers_answer_method_calls_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("pinned call columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid pinned call outcome".into()),
        };
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-types",
                "--edition=2024",
                "--cap-lints=allow",
                "--source-core",
                "minicore@pinned_calls",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "pinned_calls",
            ])
            .arg(root.join(path))
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = REJECTED
            .iter()
            .find(|(name, _)| path.ends_with(&format!("/{name}")))
            .map(|(_, reason)| *reason);
        if output.status.code() != Some(i32::from(!accepted))
            || accepted != reason.is_none()
            || reason.is_some_and(|reason| !stderr.contains(reason))
        {
            failures.push(format!("{path}: {:?} {stderr}", output.status.code()));
        }
        // Each accepted call also lowers its selected by-value receiver.
        if accepted && let Err(error) = super::program(&crate::resolve::fixture(path)?) {
            failures.push(format!("{path}: lowering: {error}"));
        }
        count += 1;
    }
    assert_eq!(count, 12);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `calls` pinned method calls, then unrelated nominals whose own pinned
/// methods no call names.
fn width(prefix: &str, calls: usize, noise: bool) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..calls {
        writeln!(
            text,
            "pub fn p{index}<T: ?Sized>(p: Pin<&mut SyncView<T>>) -> Pin<&mut T> {{ p.as_pin_mut() }}"
        )?;
    }
    for index in 0..if noise { 64 } else { 0 } {
        writeln!(
            text,
            "pub struct U{index}(u8); impl U{index} {{ pub fn pin{index}(self: Pin<&mut Self>) -> u8 {{ 0 }} }}"
        )?;
    }
    Ok(text)
}

#[test]
fn pinned_calls_track_calls_without_scanning_unrelated_receivers() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/free-generic.rs"))?;
    let prefix = source.strip_suffix(CALL).ok_or("pinned method call")?;
    let mut previous = None;
    let mut unrelated = None;
    for calls in [8, 32, 128] {
        let mut ordinary = [0; 3];
        for noise in [false, true] {
            let report =
                crate::resolve::source_provider(&width(prefix, calls, noise)?, "pinned_calls")?;
            let facts = rustgm::pipeline::analyze_source(&report)?.facts;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
            ];
            println!("pinned calls calls={calls} noise={noise} work={work:?}");
            if noise {
                // The unrelated owners validate their receivers once; no call
                // discovers them.
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
