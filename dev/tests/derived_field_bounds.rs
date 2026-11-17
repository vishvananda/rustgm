//! rustc's derives bound each field type path that begins with a type
//! parameter, such as `I::Item`, by the derived trait, after the written
//! predicates, as core's `Peekable`, `ArrayChunks` and `Intersperse` need.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/derive-projection-bounds";
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

/// Every accepted case types and lowers. Each
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
            let lowers = lowered.is_ok();
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

const OBJECT_FIELDS: &str = "pa31/tests/spec/derive-trait-object-fields";

fn object_field_phase(name: &str, view: &str) -> Result<std::process::Output> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    Ok(std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            view,
            "--cap-lints=allow",
            "--edition=2024",
            "--crate-name",
            "core",
            "--crate-disambiguator",
            "derive_object_fields",
            "--source-core",
            "core@derive_object_fields",
        ])
        .arg(root.join(OBJECT_FIELDS).join(format!("{name}.rs")))
        .output()?)
}

fn require_object_foundation(view: &str) -> Result {
    let output = object_field_phase("foundation-dyn", view)?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "foundation-dyn {view}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.stdout.is_empty(), "foundation-dyn {view}");
    Ok(())
}

#[test]
fn source_derive_object_fields_copy_actual_syntax_before_semantic_validation() -> Result {
    require_object_foundation("--emit-expanded-crate")?;
    let control = crate::resolve::fixture(&format!("{OBJECT_FIELDS}/cases.tsv"))?;
    let mut count = 0;
    for row in control.lines() {
        let (name, _) = row.split_once('\t').ok_or("derive object outcome row")?;
        let output = object_field_phase(name, "--emit-expanded-crate")?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!output.stdout.is_empty(), "{name}");
        count += 1;
    }
    assert_eq!(count, 8);
    Ok(())
}

fn inspect_object_clone_body(
    report: &rustgm::resolve::expansion::Report,
    facts: &rustgm::sema::model::Declarations,
    case: &str,
    function: &str,
) -> Result {
    use rustgm::{
        body::ir::{Owner, Rvalue},
        resolve::model::DefKind,
        types::Kind,
    };

    let definitions = &report.resolution.definitions;
    let name = |index: u32| report.package.names.get(definitions[index as usize].name);
    let program = rustgm::body::build_concrete(report, facts)?;
    let templates = rustgm::body::templates::build(report, facts)?;
    let owner = program
        .bodies
        .iter()
        .chain(templates.bodies.iter().map(|template| &template.body))
        .find(|body| {
            matches!(body.owner, Owner::Function { definition, .. }
            if definitions[definition.0 as usize].kind == DefKind::Function
                && name(definition.0) == function)
        })
        .ok_or("actual clone-use BodyIR")?;
    assert!(
        owner.values.iter().any(|value| {
            let Rvalue::Call { callee, .. } = value.kind else {
                return false;
            };
            matches!(facts.types.kind(owner.values[callee.0 as usize].ty),
            Some(Kind::FunctionItem { item, .. }) if name(item.0) == "clone")
        }),
        "{case}: missing actual Clone::clone call in {function}"
    );
    if matches!(case, "foundation-dyn" | "eh-context" | "nonstatic-object") {
        assert!(
            owner.values.iter().any(|value| {
                matches!(value.kind, Rvalue::Read(_))
                    && matches!(facts.types.kind(value.ty), Some(Kind::Adt(..)))
            }),
            "{case}: missing by-value read of the derived aggregate"
        );
    }
    Ok(())
}

fn object_field_proofs(case: &str, function: &str) -> Result<Option<String>> {
    let source = crate::resolve::fixture(&format!("{OBJECT_FIELDS}/{case}.rs"))?;
    let report = crate::resolve::source_provider(&source, "derive_object_fields")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    inspect_object_clone_body(&report, &facts, case, function)?;
    Ok(
        rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
            .err()
            .map(|error| error.to_string()),
    )
}

#[test]
fn source_derive_object_fields_keep_copy_clone_and_actual_lifetime_duties() -> Result {
    assert_eq!(
        object_field_proofs("foundation-dyn", "check_derived")?,
        None
    );
    let control = crate::resolve::fixture(&format!("{OBJECT_FIELDS}/cases.tsv"))?;
    let mut counts = [0; 2];
    for row in control.lines() {
        let (name, expected) = row.split_once('\t').ok_or("derive object outcome row")?;
        let expected: usize = expected.parse()?;
        let (diagnostic, causes) = match name {
            "captured-escape" => (
                object_field_proofs(name, "escape")?,
                &[
                    "required lifetime does not outlive",
                    "required type does not outlive",
                ][..],
            ),
            "non-dyn-compatible" => (object_field_type_error(name)?, &["dyn compatible"][..]),
            "owned-object" | "mutable-object" => (
                object_field_type_error(name)?,
                &["Copy", "Clone", "Sized"][..],
            ),
            "foundation" => (object_field_proofs(name, "setup")?, &[][..]),
            "eh-context" => (object_field_proofs(name, "duplicate")?, &[][..]),
            "nonstatic-object" | "higher-bound" => {
                (object_field_proofs(name, "preserve")?, &[][..])
            }
            _ => return Err("undeclared derive object case".into()),
        };
        assert_eq!(
            usize::from(diagnostic.is_some()),
            expected,
            "{name}: {diagnostic:?}"
        );
        if let Some(diagnostic) = diagnostic {
            assert!(
                causes.iter().any(|reason| diagnostic.contains(reason)),
                "{name}: {diagnostic}"
            );
            assert!(
                !diagnostic.contains("wider source derive profile"),
                "{name}: {diagnostic}"
            );
        }
        counts[expected] += 1;
    }
    assert_eq!(counts, [4, 4]);
    Ok(())
}

fn object_field_type_error(name: &str) -> Result<Option<String>> {
    let output = object_field_phase(name, "--emit-generic-semantics")?;
    let diagnostic = String::from_utf8(output.stderr)?;
    match output.status.code() {
        Some(0) => Ok(None),
        Some(1) => Ok(Some(diagnostic)),
        status => Err(format!("{name}: unexpected status {status:?}: {diagnostic}").into()),
    }
}
