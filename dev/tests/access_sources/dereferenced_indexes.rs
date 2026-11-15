//! An index operand reaches its indexed place through actual source Deref calls.
use crate::resolve as source;
use rustgm::sema;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/source-index-deref";

/// Each accepted case and the index expressions whose operand takes Deref calls.
const ACCEPTED: &[(&str, usize)] = &[
    ("shared.rs", 1),
    ("nested.rs", 1),
    ("references.rs", 2),
    ("slice-target.rs", 1),
    ("custom-target.rs", 1),
    ("own-index.rs", 0),
    ("mutable.rs", 5),
    ("late-mutable.rs", 1),
    ("generic.rs", 1),
    ("native.rs", 13),
];

/// Each rejected control and the reason reported for rustc's error.
const REJECTED: &[(&str, &str)] = &[
    // E0608: neither `Index` nor `Deref`.
    ("missing.rs", NO_INDEX),
    // E0596 and E0594: a mutable index through a type with `Deref` only.
    ("shared-only-borrow.rs", NO_DEREF_MUT),
    ("shared-only-store.rs", NO_DEREF_MUT),
    // E0308: the reached array, or the type's own `Index`, takes another index type.
    ("wrong-index.rs", NO_INDEX),
    ("hidden-target.rs", NO_INDEX),
    // E0596: `DerefMut` borrows an immutable binding.
    (
        "immutable-binding.rs",
        "cannot mutably borrow an immutable local",
    ),
];
/// Each recorded gap, which rustc accepts, and the reason it stops for.
const GAPS: &[(&str, &str)] = &[
    // rustc selects the reference's own `Index` before any dereference.
    (
        "reference-impl.rs",
        "an index implementation for a reference type is not selected",
    ),
    // A `ref mut` binding does not yet make its index place mutable.
    (
        "ref-mut-binding.rs",
        "cannot mutably borrow shared or static storage",
    ),
];
/// The indexed type has no index implementation for the index type.
const NO_INDEX: &str = "associated type requires an unambiguous validated implementation";
/// The mutable operand step has no `DerefMut` implementation to call.
const NO_DEREF_MUT: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";

fn cases() -> Result<Vec<(String, bool)>> {
    let mut cases = Vec::new();
    for line in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("index case columns")?;
        let name = path.rsplit('/').next().ok_or("index case name")?;
        cases.push((name.to_owned(), expected == "0"));
    }
    assert_eq!(cases.len(), ACCEPTED.len() + GAPS.len() + REJECTED.len());
    Ok(cases)
}

fn consume(name: &str) -> std::result::Result<(), String> {
    let text = source::fixture(&format!("{PACKET}/{name}")).map_err(|e| e.to_string())?;
    let report = source::source_provider(&text, "index_deref")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
    rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
        .map_err(|e| e.to_string())
}

#[test]
fn index_operands_keep_their_deref_calls_beside_the_index_operation() -> Result {
    let mut failures = Vec::new();
    for &(name, expected) in ACCEPTED {
        let text = source::fixture(&format!("{PACKET}/{name}"))?;
        let report = source::source_provider(&text, "index_semantics")?;
        let facts = match sema::declarations::analyze(&report)
            .and_then(|facts| sema::bodies::analyze(&report, facts))
        {
            Ok(facts) => facts,
            Err(error) => {
                failures.push(format!("{name}: {error}"));
                continue;
            }
        };
        let mut view = Vec::new();
        sema::type_view::write(&report, &facts, &mut view)?;
        let plans = String::from_utf8(view)?
            .lines()
            .filter(|line| line.starts_with("index-operand "))
            .count();
        if plans != expected {
            failures.push(format!(
                "{name}: {plans} index operand plans, expected {expected}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn index_operand_calls_preserve_mutability_index_type_and_binding_requirements() -> Result {
    let mut failures = Vec::new();
    for (name, accepted) in cases()? {
        let reason = |rows: &[(&str, &'static str)]| {
            rows.iter()
                .find(|(row, _)| *row == name)
                .map(|(_, reason)| *reason)
        };
        let rejected = reason(REJECTED);
        assert_eq!(accepted, rejected.is_none(), "{name}");
        match (rejected.or_else(|| reason(GAPS)), consume(&name)) {
            (None, Ok(())) => {}
            (Some(reason), Err(error)) if error.contains(reason) => {}
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|e| e, |()| "accepted".into())
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn index_operand_calls_run_once_in_order_and_reach_native() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/index-deref/native");
    std::fs::create_dir_all(&folder)?;
    let text = source::fixture(&format!("{PACKET}/native.rs"))?;
    let report = source::source_provider(&text, "index_native")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    crate::ffi_program::check(
        &root,
        &folder,
        "native",
        &program,
        &source::fixture(&format!("{PACKET}/observer.c"))?,
        &source::fixture(&format!("{PACKET}/native.stdout"))?,
    )?;
    Ok(())
}
