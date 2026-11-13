//! A source comparison's `Self` is its left operand's type with fresh regions,
//! so the `Rhs` selected beneath the method signature's binder names no body
//! lifetime parameter, as core's derived `eq` of `marker/variance.rs:107`
//! needs. An operand left in its own type is borrowed in that type and related
//! to the parameter, as rustc's borrow checker relates renumbered regions.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/comparison-body-lifetimes";
const OUTLIVES: &str = "required lifetime does not outlive its target";
const SUFFIX: &str =
    "    pub fn same<'a>(a: Wrap<&'a ()>, b: Wrap<&'a ()>) -> bool { a == b }\n}\n";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "comparison_lifetimes")?;
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
            "minicore@comparison_lifetimes",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "comparison_lifetimes",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

/// Every case types. rustc's borrow checker rejects the two invariant
/// comparisons of distinct lifetimes, which fail prepared lowering's lifetime
/// proof here; every other case lowers.
#[test]
fn comparison_goals_keep_body_lifetimes_free_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("comparison lifetime columns")?;
        let (types, stderr) = typed(&root.join(path))?;
        let lowered = program(&crate::resolve::fixture(path)?)
            .map(|_| ())
            .map_err(|error| error.to_string());
        let matches = types
            && match expected {
                "0" => lowered.is_ok(),
                "1" => lowered
                    .as_ref()
                    .is_err_and(|error| error.contains(OUTLIVES)),
                _ => return Err("invalid comparison lifetime outcome".into()),
            };
        if !matches {
            failures.push(format!(
                "{path}: types={types} {stderr} lowered={lowered:?}"
            ));
        }
        count += 1;
    }
    assert_eq!(count, 21);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `functions` functions, each comparing two operands with a body lifetime.
fn width(prefix: &str, functions: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..functions {
        writeln!(
            text,
            "    pub fn same{index}<'a>(a: Wrap<&'a ()>, b: Wrap<&'a ()>) -> bool {{ a == b }}"
        )?;
    }
    text.push_str("}\n");
    Ok(text)
}

/// Typing work stays linear in the comparisons, and every width lowers.
#[test]
fn comparison_lifetime_work_tracks_functions() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/local-values.rs"))?;
    let prefix = source
        .strip_suffix(SUFFIX)
        .ok_or("comparison lifetime suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for functions in [8, 32, 128] {
        let text = width(prefix, functions)?;
        let report = crate::resolve::source_provider(&text, "comparison_lifetimes")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("comparison lifetimes functions={functions} work={work:?}");
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
