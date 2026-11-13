//! A body that opens its declaration's lifetimes keeps the declaration's
//! certified projection premises, so core's `SplitInclusive` `Clone` proves
//! `<P as Pattern>::Searcher<'a>: Clone` for the impl's fresh region and
//! leaves the region equality to the lifetime proof, as rustc does.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/gat-region-premises";
const LOCAL: &str = "pub fn copy_split<'a, P: Pattern<Searcher<'a>: Clone>>(s: &SplitInternal<'a, P>) -> SplitInternal<'a, P> {\n    s.clone()\n}\n";
const UNSATISFIED: &str = "no applicable method for this receiver";
const OUTLIVES: &str = "required lifetime does not outlive its target";
/// rustc reports E0599 for the missing bounds. Its borrow checker rejects the
/// other three ("lifetime may not live long enough"), which type here and
/// fail prepared lowering's lifetime proof.
const REJECTED: [(&str, &str, bool); 5] = [
    ("missing-bound.rs", UNSATISFIED, false),
    ("wrapper-missing-bound.rs", UNSATISFIED, false),
    ("other-lifetime.rs", OUTLIVES, true),
    ("static-premise.rs", OUTLIVES, true),
    ("outlives-only.rs", OUTLIVES, true),
];

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "gat_regions")?;
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
            "minicore@gat_regions",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "gat_regions",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

#[test]
fn opened_gat_premises_prove_fresh_region_goals_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("gat region columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid gat region outcome".into()),
        };
        let reason = REJECTED
            .iter()
            .find(|(name, ..)| path.ends_with(&format!("/{name}")))
            .map(|&(_, reason, types)| (reason, types));
        let (types, stderr) = typed(&root.join(path))?;
        let outcome = if types {
            program(&crate::resolve::fixture(path)?)
                .err()
                .map(|error| error.to_string())
        } else {
            Some(stderr)
        };
        let matches = match (&outcome, reason) {
            (None, None) => accepted,
            (Some(message), Some((reason, typing))) => {
                !accepted && types == typing && message.contains(reason)
            }
            _ => false,
        };
        if !matches {
            failures.push(format!("{path}: types={types} {outcome:?}"));
        }
        count += 1;
    }
    assert_eq!(count, 15);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `functions` free functions, each cloning a split through its own premise.
fn width(prefix: &str, functions: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..functions {
        writeln!(
            text,
            "pub fn copy{index}<'a, P: Pattern<Searcher<'a>: Clone>>(s: &SplitInternal<'a, P>) -> SplitInternal<'a, P> {{ s.clone() }}"
        )?;
    }
    Ok(text)
}

/// Typing work stays linear, and every width completes prepared lowering.
#[test]
fn gat_region_premise_work_tracks_functions() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/local-value.rs"))?;
    let prefix = source.strip_suffix(LOCAL).ok_or("gat region suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for functions in [8, 32, 128] {
        let text = width(prefix, functions)?;
        let report = crate::resolve::source_provider(&text, "gat_regions")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("gat region premises functions={functions} work={work:?}");
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
