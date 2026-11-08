//! Consumed original bodies pay their retained rows once; shrinking the
//! consumed containers pays work only, so lowering many small bodies stays
//! below the header ledger's input-graph bound.
use super::*;
use crate::sema::header_attributes::{source_contract_bound, source_contract_costs};
use std::fmt::Write;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
type Item = fn(&mut String, usize) -> std::fmt::Result;

const WORK: &str = "pa31/tests/spec/needs-drop/work";
const BASE: &str = "pa31/tests/spec/needs-drop/work/width8-calls8.rs";
const ROOT: &str = "struct Root(Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf);\n";

/// The `needs_drop` scaling base with `count` numbered items after its root
/// nominal, where the declared noise inputs place their unrelated nominals.
fn inserted(count: usize, item: Item) -> Result<String> {
    let base = crate::test_source::fixture(BASE)?;
    let at = base.find(ROOT).ok_or("scaling root")? + ROOT.len();
    let mut source = base[..at].to_owned();
    for index in 0..count {
        item(&mut source, index)?;
    }
    source.push_str(&base[at..]);
    Ok(source)
}
fn noise(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(source, "struct Noise{index}([u8;{}]);", index + 1)
}
fn constant(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(source, "const C{index}:u8=0;")
}
fn unnamed(source: &mut String, _: usize) -> std::fmt::Result {
    writeln!(source, "const _:u8=0;")
}
fn alias(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(source, "type A{index}=[u8;0];")
}

/// The header ledger's byte bound, its bytes after analysis and its bytes
/// after lowering every root of `source`.
fn lowered(source: &str) -> Result<[usize; 3]> {
    let report = crate::test_source::source_provider(source, "needs_drop_work")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let bound = source_contract_bound(&facts).1;
    let analysis = source_contract_costs(&facts).1;
    crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok([bound, analysis, source_contract_costs(&facts).1])
}

/// Bytes paid per added input-graph element, in total and by lowering alone,
/// between `small` and `large` copies of `item`.
fn per_element(item: Item, small: usize, large: usize) -> Result<(usize, usize)> {
    let [bound0, analysis0, total0] = lowered(&inserted(small, item)?)?;
    let [bound1, analysis1, total1] = lowered(&inserted(large, item)?)?;
    let elements = (bound1 - bound0) / 128;
    let lowering = (total1 - analysis1) - (total0 - analysis0);
    eprintln!(
        "row bytes {small}->{large}: bound {bound0}->{bound1} analysis {analysis0}->{analysis1} total {total0}->{total1} elements +{elements} lowering +{lowering}"
    );
    Ok(((total1 - total0) / elements, lowering / elements))
}

#[test]
fn declared_scaling_inputs_lower_inside_their_input_bound() -> Result {
    // The native scaling test lowers these same inputs; this private control
    // keeps their header-ledger margins visible without the backend.
    let mut names = Vec::new();
    for width in [8, 32, 128] {
        for calls in [8, 32, 128] {
            names.push(format!("width{width}-calls{calls}"));
        }
    }
    names.extend(
        [
            "width8-calls8-noise64",
            "width8-calls8-noise1024",
            "ordinary-unused-intrinsic",
            "primitive-calls128",
        ]
        .map(str::to_owned),
    );
    for name in names {
        let source = crate::test_source::fixture(&format!("{WORK}/{name}.rs"))?;
        let [bound, analysis, total] = lowered(&source)?;
        eprintln!(
            "declared {name}: analysis {analysis} total {total} of {bound} ({}%)",
            total * 100 / bound
        );
        assert!(total < bound, "{name}: {total} of {bound}");
    }
    Ok(())
}

#[test]
fn unrelated_constant_bodies_pay_less_than_their_input_bound_grants() -> Result {
    for count in [64, 1024] {
        let declared =
            crate::test_source::fixture(&format!("{WORK}/width8-calls8-noise{count}.rs"))?;
        assert_eq!(inserted(count, noise)?, declared, "{count}");
    }
    let rows =
        [64, 256, 1024].map(|count| inserted(count, noise).and_then(|source| lowered(&source)));
    let [small, middle, large] = rows;
    let (small, middle, large) = (small?, middle?, large?);
    eprintln!("noise rows 64={small:?} 256={middle:?} 1024={large:?}");
    // Each added nominal pays less than the 11 input elements it adds grant,
    // and the increments stay linear apart from row capacity doubling.
    for (before, after, added) in [(small, middle, 192), (middle, large, 768)] {
        assert!(
            after[2] - before[2] < after[0] - before[0],
            "{before:?} {after:?}"
        );
        let lowering = (after[2] - after[1]) - (before[2] - before[1]);
        assert!(lowering < after[0] - before[0], "{before:?} {after:?}");
        assert!((after[2] - before[2]) / added <= 2 * (middle[2] - small[2]) / 192);
    }
    assert!(large[2] < large[0], "{large:?}");
    Ok(())
}

#[test]
fn literal_constant_items_pay_less_per_element_than_their_input_bound() -> Result {
    // Literal constant items are the densest bodies measured here: five input
    // elements each, and all their 115 bytes per element are lowering rows.
    for (name, item) in [
        ("noise", noise as Item),
        ("constant", constant),
        ("unnamed", unnamed),
        ("alias", alias),
    ] {
        let (total, lowering) = per_element(item, 256, 1024)?;
        eprintln!("row bytes per element {name}: total {total} lowering {lowering} of 128");
        assert!(total < 128, "{name}: {total} bytes per element");
    }
    Ok(())
}

#[test]
fn consumed_originals_pay_their_rows_once_and_shrinking_pays_only_work() -> Result {
    let source = inserted(1024, noise)?;
    let report = crate::test_source::source_provider(&source, "needs_drop_work")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let mut program = crate::body::build_concrete(&report, &facts)?;
    let count = program.bodies.len();
    let built = retained::vector(&program.drops.as_ref().ok_or("pending rows")?.bodies);
    let before = source_contract_costs(&facts);
    let mut originals = Originals::take(&report, &facts, &mut program)?;
    let taken = source_contract_costs(&facts);
    // Retained growth stays paid: the row owner, the replacement row vector
    // and the moved one, which no earlier owner charged when it was built.
    let replacement = Growth::vector::<Requirements>(0, 0, count).bytes;
    assert_eq!(
        taken.1 - before.1,
        size_of::<Rows<'_>>() + replacement + built
    );
    originals.reverse(&facts)?;
    let reversed = source_contract_costs(&facts);
    let (mut copied, mut recharged, mut consumed, mut shrinks) = (0, 0, 0, 0);
    while originals.pop()?.is_some() {
        consumed += 1;
        let rows = originals.requirements.as_ref().ok_or("rows")?.values.len();
        if originals.bodies.len() < originals.bodies.capacity() / 2 {
            copied += originals.bodies.len() + rows;
            shrinks += 1;
            recharged +=
                originals.bodies.len() * size_of::<Body>() + rows * size_of::<Requirements>();
        }
        originals.shrink(&facts)?;
    }
    let shrunk = source_contract_costs(&facts);
    eprintln!(
        "originals count={count} built={built} take={} copied={copied} recharged={recharged} body={} row={}",
        taken.1 - before.1,
        size_of::<Body>(),
        size_of::<Requirements>()
    );
    assert_eq!(consumed, count);
    assert_eq!(shrunk.1, reversed.1, "shrinking retains no new storage");
    // Each moved body and row pays one unit, and each charge one more to
    // authenticate its source payer.
    assert_eq!(shrunk.0 - reversed.0, copied + shrinks);
    // Before the repair every halving paid its copy again as storage, at
    // least half of the consumed body and row storage a second time.
    assert!(recharged >= (count / 2 - 1) * (size_of::<Body>() + size_of::<Requirements>()));
    Ok(())
}
