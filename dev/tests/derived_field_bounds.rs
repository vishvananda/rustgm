//! rustc's derives bound each field type path that begins with a type
//! parameter, such as `I::Item`, by the derived trait, after the written
//! predicates, as core's `Peekable`, `ArrayChunks` and `Intersperse` need.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/derive-projection-bounds";
/// Accepted cases that type and stop in prepared lowering at a recorded gap.
/// A manual `Debug` impl over the same struct stops there too
/// (`obj/parallel/derive-projections/core/batch3`): an impl instance reading
/// a projected field of a struct with a lifetime parameter.
const LOWERING_GAPS: [(&str, &str); 1] = [(
    "reference-field.rs",
    "generic aggregate requires instantiated fields",
)];
const SUFFIX: &str = "    #[derive(Clone, Debug)]
    pub struct Wrap<T> { value: T }
    pub fn again(value: &Wrap<u8>) -> Wrap<u8> { Clone::clone(value) }
}
";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "derive_projections")?;
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
            "minicore@derive_projections",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "derive_projections",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

/// Every accepted case types and lowers, except for the recorded gaps. Each
/// rejection reports its message at the last occurrence of its site: a use the
/// derived predicate refuses, the derive whose body needs an unbounded
/// qualified path, or a type macro.
#[test]
fn derived_field_paths_are_bounded_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let mut columns = line.split('\t');
        let path = columns.next().ok_or("derive bound path")?;
        let expected = columns.next().ok_or("derive bound outcome")?;
        let (types, stderr) = typed(&root.join(path))?;
        let source = crate::resolve::fixture(path)?;
        let matches = if expected == "0" {
            let lowered = program(&source)
                .map(|_| ())
                .map_err(|error| error.to_string());
            let gap = LOWERING_GAPS
                .iter()
                .find(|(case, _)| path.ends_with(case))
                .map(|(_, message)| *message);
            let lowers = match gap {
                Some(message) => lowered.as_ref().is_err_and(|error| error.contains(message)),
                None => lowered.is_ok(),
            };
            if !lowers {
                failures.push(format!("{path}: lowering {lowered:?}"));
            }
            types && lowers
        } else {
            let site = columns.next().ok_or("derive bound rejection site")?;
            let offset = source.rfind(site).ok_or("derive bound site text")?;
            !types && stderr.starts_with(&format!("ERROR: file 0: {offset}: {expected}\n"))
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

/// One derived `Clone` over `fields` fields, each of type `Opt<I::Item>`,
/// so its impl repeats `I::Item: Clone` once per field, as rustc's does.
fn width(prefix: &str, fields: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    text.push_str("    #[derive(Clone)]\n    pub struct Wide<I: Iter> {");
    for index in 0..fields {
        write!(text, " f{index}: Opt<I::Item>,")?;
    }
    text.push_str(
        " }\n    pub fn again(value: &Wide<Counter>) -> Wide<Counter> { Clone::clone(value) }\n}\n",
    );
    Ok(text)
}

/// Typing work stays linear in the bounded fields, and every width lowers.
#[test]
fn derived_field_bound_work_tracks_fields() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/sole-parameter.rs"))?;
    let prefix = source.strip_suffix(SUFFIX).ok_or("derive bound suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for fields in [8, 32, 128] {
        let text = width(prefix, fields)?;
        let report = crate::resolve::source_provider(&text, "derive_projections")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("derived field bounds fields={fields} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 4 * old + 512,
                    "fields={fields}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
        program(&text)?;
    }
    Ok(())
}
