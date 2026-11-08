//! Coherence pairs only headers in one simplified self bucket, and a pair whose
//! trait references cannot unify pays its comparison, never an instantiation.
//! The reject is conservative: no pair it answers has unifying headers.
use super::{AUDIT, possible};
use crate::{
    resolve::model::DefId,
    sema::{
        model::Declarations,
        traits::implementations::{Implementation, Implementations},
    },
    types::{ItemId, parameters::predicates::Predicate},
};
use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const COUNTS: [usize; 3] = [6, 24, 96];
const WIDTHS: [usize; 2] = [1, 8];
/// Every PA31 packet whose impls coherence or specialization compares.
const CONTROLS: [&str; 12] = [
    "callable-sugar-coherence",
    "deep-reject",
    "determined-parents",
    "fn-pointer-coherence",
    "fundamental-policy",
    "impl-header-projections",
    "lifetime-specialization",
    "negative-coherence",
    "source-reservation-impl",
    "specialization-fixpoint",
    "specialization-graph",
    "specialization-selection",
];
/// Each deep-reject control: whether rustc 1.98.0 accepts it (`outcomes.json`)
/// and how many of its pairs the reject answers. A difference below the depth
/// limit, behind an alias or under a parameter's projection reaches the complete
/// equation. A closed projection is already normalized in the formed header.
const DEEP_REJECT: [(&str, bool, usize); 19] = [
    ("nested-nominals", true, 6),
    ("trait-arguments", true, 1),
    ("array-lengths", true, 1),
    ("reference-mutability", true, 1),
    ("pointer-mutability", true, 1),
    ("tuple-arity", true, 1),
    ("function-arity", true, 1),
    ("function-safety", true, 1),
    ("object-principals", true, 1),
    ("past-depth", true, 0),
    ("parameter", false, 0),
    ("crossed-arguments", false, 0),
    ("lifetime", false, 0),
    ("const-parameter", false, 0),
    ("past-depth-overlap", false, 0),
    ("alias-overlap", false, 0),
    ("projection-overlap", false, 0),
    ("parameter-projection", false, 0),
    ("projection-disjoint", true, 1),
];

/// `count` impls of one trait whose headers carry `width` parameters. Nested
/// subjects share the simplified self type `W` and differ one level down.
fn analyze(count: usize, width: usize, nested: bool) -> Result<(Declarations, ItemId)> {
    let parameters = (0..width)
        .map(|index| format!("P{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let mut source = String::from("trait A<U>{}struct W<T>(T);");
    for index in 0..count {
        write!(source, "struct S{index}<T>(T);")?;
        let subject = if nested {
            format!("W<S{index}<u8>>")
        } else {
            format!("S{index}<u8>")
        };
        write!(
            source,
            "impl<{parameters}>A<({parameters},)>for {subject}{{}}"
        )?;
    }
    let report = crate::test_source::resolve(&source, true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let trait_item = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "A")
        .map(u32::try_from)
        .transpose()?
        .map(ItemId)
        .ok_or("actual trait A")?;
    Ok((facts, trait_item))
}

fn headers(facts: &Declarations, item: ItemId) -> Result<Vec<(Predicate, DefId)>> {
    Ok(facts
        .generics
        .implementations
        .as_ref()
        .ok_or("registry")?
        .headers()
        .filter(|header| {
            header
                .predicate
                .bound
                .trait_parts()
                .and_then(|(id, _)| facts.types.trait_application(id))
                .is_some_and(|application| application.item == item)
        })
        .map(|header| (header.predicate, header.owner))
        .collect())
}

/// The pairs the coherence index forms for these headers, from a fresh registry.
fn indexed_pairs(facts: &mut Declarations, item: ItemId) -> Result<Vec<[(Predicate, DefId); 2]>> {
    let mut registry = Implementations::default();
    for (predicate, owner) in headers(facts, item)? {
        registry.prepare(
            &mut facts.types,
            item,
            Implementation {
                owner,
                source: None,
                predicate,
                methods: Vec::new(),
                types: Vec::new(),
                constants: Vec::new(),
            },
        )?;
    }
    Ok(registry
        .take_overlap_pairs()
        .into_iter()
        .map(|(a, owner_a, b, owner_b)| [(a, owner_a), (b, owner_b)])
        .collect())
}

#[test]
fn distinct_simplified_self_types_form_no_coherence_pairs() -> Result {
    for count in COUNTS {
        for width in WIDTHS {
            let (mut facts, item) = analyze(count, width, false)?;
            assert_eq!(headers(&facts, item)?.len(), count);
            assert!(
                indexed_pairs(&mut facts, item)?.is_empty(),
                "{count} x {width}"
            );
        }
    }
    Ok(())
}

#[test]
fn unrelated_nested_subjects_pay_their_comparison_and_no_instantiation() -> Result {
    let mut rejected = Vec::new();
    let mut overlapping = Vec::new();
    for count in COUNTS {
        for width in WIDTHS {
            let (mut facts, item) = analyze(count, width, true)?;
            let pairs = indexed_pairs(&mut facts, item)?;
            assert_eq!(pairs.len(), count * (count - 1) / 2, "{count} x {width}");
            for [first, second] in pairs {
                let (intersects, work) = possible(&mut facts, first, second)?;
                assert!(!intersects, "distinct nominal arguments cannot unify");
                rejected.push(work);
            }
            // Control: a header against its own copy unifies and pays the real
            // instantiation, which grows with the header's parameters.
            let header = *headers(&facts, item)?.first().ok_or("header")?;
            let (intersects, work) = possible(&mut facts, header, header)?;
            assert!(intersects);
            overlapping.push((width, work));
        }
    }
    let constant = rejected.first().copied().ok_or("rejected pairs")?;
    assert!(
        rejected.iter().all(|&work| work == constant),
        "rejected pair work depends on the crate or header size"
    );
    let least = overlapping
        .iter()
        .map(|&(_, work)| work)
        .min()
        .ok_or("controls")?;
    assert!(constant < least, "{constant} vs {overlapping:?}");
    assert!(
        overlapping
            .chunks(WIDTHS.len())
            .all(|controls| controls.windows(2).all(|pair| pair[0].1 < pair[1].1)),
        "{overlapping:?}"
    );
    Ok(())
}

fn sources(directory: &Path, found: &mut Vec<PathBuf>) -> Result {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            sources(&path, found)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
    Ok(())
}

/// Declaration analysis with the audit on: whether it accepted, then the pairs
/// compared, the pairs rejected and the rejected pairs that still unify.
fn audited(source: &str) -> Result<(bool, [usize; 3])> {
    AUDIT.with(|cell| cell.set(Some([0; 3])));
    let report = if source.contains("#![no_core]") {
        crate::test_source::source_provider(source, "audited")
    } else {
        crate::test_source::resolve(source, true)
    };
    // A control rejected before coherence still reports its audited pairs.
    let accepted = report.is_ok_and(|report| crate::sema::declarations::analyze(&report).is_ok());
    let counts = AUDIT.with(std::cell::Cell::take).ok_or("audit")?;
    Ok((accepted, counts))
}

#[test]
fn deep_reject_controls_reject_rigid_differences_and_keep_hidden_ones() -> Result {
    for (name, accepted, rejected) in DEEP_REJECT {
        let path = format!("pa31/tests/spec/deep-reject/{name}.rs");
        let (outcome, [pairs, answered, unifying]) = audited(&crate::test_source::fixture(&path)?)?;
        assert_eq!(outcome, accepted, "{name}");
        assert_eq!(answered, rejected, "{name}: {pairs} pairs");
        assert!(pairs >= rejected.max(1), "{name}: {pairs} pairs");
        assert_eq!(unifying, 0, "{name}");
    }
    Ok(())
}

#[test]
fn rejected_pairs_never_unify_in_any_coherence_or_specialization_control() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../pa31/tests/spec");
    let mut found = Vec::new();
    for control in CONTROLS {
        sources(&root.join(control), &mut found)?;
    }
    found.sort();
    let mut total = [0_usize; 3];
    let mut compared = 0_usize;
    for path in &found {
        let (_, counts) = audited(&std::fs::read_to_string(path)?)?;
        assert_eq!(counts[2], 0, "{}: a rejected pair unifies", path.display());
        compared += usize::from(counts[0] != 0);
        for (sum, count) in total.iter_mut().zip(counts) {
            *sum += count;
        }
    }
    // The audit is not vacuous: controls compare pairs, and some are rejected.
    assert!(compared > 0, "{compared} of {} {total:?}", found.len());
    assert!(total[1] > 0 && total[1] < total[0], "{total:?}");
    Ok(())
}
