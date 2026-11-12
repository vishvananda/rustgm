//! A destructuring assignment is rustc's `{ let <pattern> = rhs; leaf = binding; ... }`:
//! core's `(left, right, dst) = merge_up(...)` stores each field to its place leaf.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/destructuring-assignment";
/// Typing rejections come from the CLI; ownership ones from prepared lowering.
const REJECTED: [(&str, &str); 6] = [
    ("literal-leaf.rs", "assignment requires a place"),
    ("arity.rs", "incompatible types"),
    ("compound.rs", "assignment requires a place"),
    (
        "immutable-leaf.rs",
        "cannot assign twice to an immutable local",
    ),
    (
        "borrowed-source.rs",
        "cannot move a non-Copy value out of a reference",
    ),
    (
        "moved-source.rs",
        "local may be uninitialized or moved at this read",
    ),
];
const SWAP: &str = "pub fn swap(mut a: u8, mut b: u8) -> u8 {\n    (a, b) = (b, a);\n    a\n}\n";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "destructuring")?;
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

/// The CLI's typing status, its stderr and its `destructure` operation count.
fn typed(path: &std::path::Path) -> Result<(bool, String, usize)> {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-types",
            "--edition=2024",
            "--cap-lints=allow",
            "--source-core",
            "minicore@destructuring",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "destructuring",
        ])
        .arg(path)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let operations = stdout
        .lines()
        .filter(|line| line.starts_with("operation ") && line.ends_with(" destructure"))
        .count();
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        operations,
    ))
}

#[test]
fn destructuring_assignments_store_each_leaf_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("destructuring columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid destructuring outcome".into()),
        };
        let reason = REJECTED
            .iter()
            .find(|(name, _)| path.ends_with(&format!("/{name}")))
            .map(|(_, reason)| *reason);
        let (types, stderr, operations) = typed(&root.join(path))?;
        let outcome = if types {
            program(&crate::resolve::fixture(path)?)
                .err()
                .map(|error| error.to_string())
        } else {
            Some(stderr)
        };
        let matches = match (&outcome, reason) {
            (None, None) => accepted,
            (Some(message), Some(reason)) => !accepted && message.contains(reason),
            _ => false,
        };
        if !matches {
            failures.push(format!("{path}: {outcome:?}"));
        }
        if accepted && operations == 0 {
            failures.push(format!("{path}: no destructure operation"));
        }
        count += 1;
    }
    assert_eq!(count, 17);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `calls` functions, each swapping two locals by destructuring assignment.
fn width(prefix: &str, calls: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..calls {
        writeln!(
            text,
            "pub fn swap{index}(mut a: u8, mut b: u8) -> u8 {{ (a, b) = (b, a); a }}"
        )?;
    }
    Ok(text)
}

#[test]
fn destructuring_work_tracks_assignments() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/swap.rs"))?;
    let prefix = source.strip_suffix(SWAP).ok_or("destructuring swap")?;
    let mut previous: Option<[usize; 3]> = None;
    for calls in [8, 32, 128] {
        let report = crate::resolve::source_provider(&width(prefix, calls)?, "destructuring")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("destructuring swaps calls={calls} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 4 * old + 512,
                    "calls={calls}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
    }
    Ok(())
}
