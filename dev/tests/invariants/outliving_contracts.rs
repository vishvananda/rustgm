//! Demanded source nominal requirements use the same full inference semantics.
use super::*;
use crate::{
    resolve::model::{DefId, DefKind},
    sema::model::Declarations,
};
use std::{cell::Cell, fmt::Write};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn item(report: &crate::resolve::expansion::Report, name: &str) -> TestResult<ItemId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| {
            definition.kind == DefKind::Struct && report.package.names.get(definition.name) == name
        })
        .ok_or("actual nominal declaration")?;
    Ok(ItemId(u32::try_from(index)?))
}
fn append(
    values: &mut Bounds,
    facts: &mut Declarations,
    root: ItemId,
    pay: Payment<'_>,
) -> TestResult {
    let mut explicit: HashMap<ItemId, Vec<Pair>> = HashMap::default();
    for &(item, pair) in &facts.lifetimes.nominal_bounds {
        explicit.entry(item).or_default().push(pair);
    }
    let predicates = &facts.generics.predicates;
    values.append_reached(
        &mut facts.types,
        &facts.aggregates,
        root,
        &explicit,
        |item| {
            predicates
                .get(&DefId(item.0))
                .into_iter()
                .flatten()
                .map(|predicate| predicate.value)
        },
        pay,
    )?;
    Ok(())
}
fn same(values: &Bounds, full: &Bounds, item: ItemId) -> TestResult {
    let mut actual = values.get(item).ok_or("reached requirements")?.to_vec();
    let mut expected = full.get(item).ok_or("full requirements")?.to_vec();
    actual.sort_unstable_by_key(|(a, b)| (a.0, b.0));
    expected.sort_unstable_by_key(|(a, b)| (a.0, b.0));
    assert_eq!(actual, expected);
    let mut actual = values
        .parameters(item)
        .ok_or("reached parameters")?
        .to_vec();
    let mut expected = full.parameters(item).ok_or("full parameters")?.to_vec();
    actual.sort_unstable_by_key(|(parameter, region)| (parameter.0, region.0));
    expected.sort_unstable_by_key(|(parameter, region)| (parameter.0, region.0));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn reached_cycles_and_declared_static_bounds_match_full_inference() -> TestResult {
    let report = crate::test_source::resolve(
        "struct Link<'a,'b>{next:*const Link<'a,'b>,value:&'a &'b u8}\
         struct Wrap<'a,'b>{link:Link<'a,'b>}\
         struct Fixed<'a:'static,T:'a>{value:&'static &'a T}",
        true,
    )?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let mut values = Bounds::default();
    let wrap = item(&report, "Wrap")?;
    append(&mut values, &mut facts, wrap, &|_, _| Ok(()))?;
    for name in ["Link", "Wrap"] {
        same(
            &values,
            &facts.lifetimes.nominal_requirements,
            item(&report, name)?,
        )?;
    }
    assert_eq!(values.items.len(), 2);
    assert!(values.get(item(&report, "Fixed")?).is_none());
    let fixed = item(&report, "Fixed")?;
    append(&mut values, &mut facts, fixed, &|_, _| Ok(()))?;
    same(&values, &facts.lifetimes.nominal_requirements, fixed)?;
    assert!(!values.get(fixed).ok_or("fixed requirements")?.is_empty());
    assert!(
        !values
            .parameters(fixed)
            .ok_or("fixed type requirements")?
            .is_empty()
    );
    Ok(())
}

#[test]
fn failed_scratch_and_work_payments_publish_no_nominal_requirements() -> TestResult {
    let report = crate::test_source::resolve(
        "struct Link<'a,'b>{next:*const Link<'a,'b>,value:&'a &'b u8}",
        true,
    )?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let root = item(&report, "Link")?;
    let mut values = Bounds::default();
    let attempted = Cell::new((0_usize, 0_usize));
    assert!(
        append(&mut values, &mut facts, root, &|work, bytes| {
            let old = attempted.get();
            attempted.set((old.0 + work, old.1 + bytes));
            if bytes == 0 {
                Ok(())
            } else {
                Err("test scratch refused")
            }
        })
        .is_err()
    );
    assert!(attempted.get().0 > 0 && attempted.get().1 > 0);
    assert!(values.items.is_empty());
    assert_eq!(values.steps, 0);
    let attempted = Cell::new((0_usize, 0_usize));
    assert!(
        append(&mut values, &mut facts, root, &|work, bytes| {
            let old = attempted.get();
            let next = (old.0 + work, old.1 + bytes);
            attempted.set(next);
            if next.0 <= 20 {
                Ok(())
            } else {
                Err("test work refused")
            }
        })
        .is_err()
    );
    assert!(attempted.get().0 > 20 && attempted.get().1 > 0);
    assert!(values.items.is_empty());
    assert_eq!(values.steps, 0);
    Ok(())
}

#[test]
fn shared_wide_cached_contracts_are_linear_and_warm_queries_allocate_nothing() -> TestResult {
    for width in [16, 64, 256] {
        let mut source = String::from("struct Leaf<'a,'b>{");
        for index in 0..width {
            write!(source, "f{index}:&'a &'b u8,")?;
        }
        source.push('}');
        for index in 0..width {
            write!(source, "struct Wrap{index}<'a,'b>{{leaf:Leaf<'a,'b>}}")?;
        }
        let report = crate::test_source::resolve(&source, true)?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let mut values = Bounds::default();
        let paid = Cell::new((0_usize, 0_usize));
        let pay = |work, bytes| {
            let old = paid.get();
            paid.set((old.0 + work, old.1 + bytes));
            Ok(())
        };
        for index in 0..width {
            let root = item(&report, &format!("Wrap{index}"))?;
            append(&mut values, &mut facts, root, &pay)?;
            same(&values, &facts.lifetimes.nominal_requirements, root)?;
        }
        assert_eq!(values.items.len(), width + 1);
        assert!(
            paid.get().0 < width * 400 + 100,
            "{width}: {}",
            paid.get().0
        );
        let before = paid.get();
        let steps = values.steps;
        let root = item(&report, "Wrap0")?;
        for _ in 0..width {
            append(&mut values, &mut facts, root, &pay)?;
        }
        assert_eq!(paid.get(), (before.0 + width, before.1));
        assert_eq!(values.steps, steps);
    }
    Ok(())
}

#[test]
fn one_wide_leaf_then_many_small_owners_retires_sparse_clear_capacity() -> TestResult {
    for width in [16, 64, 256] {
        let mut source = String::from("struct Leaf<'a,'b>{");
        for index in 0..width {
            write!(source, "f{index}:[&'a &'b u8;{index}],")?;
        }
        source.push('}');
        for index in 0..width {
            write!(source, "struct Wrap{index}<'a,'b>{{leaf:Leaf<'a,'b>}}")?;
        }
        source.push_str("struct Bundle<'a,'b>{");
        for index in 0..width {
            write!(source, "w{index}:Wrap{index}<'a,'b>,")?;
        }
        source.push('}');
        let report = crate::test_source::resolve(&source, true)?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let mut values = Bounds::default();
        let paid = Cell::new((0_usize, 0_usize));
        append(
            &mut values,
            &mut facts,
            item(&report, "Bundle")?,
            &|work, bytes| {
                let before = paid.get();
                paid.set((before.0 + work, before.1 + bytes));
                Ok(())
            },
        )?;
        assert_eq!(values.items.len(), width + 2);
        same(
            &values,
            &facts.lifetimes.nominal_requirements,
            item(&report, "Bundle")?,
        )?;
        println!(
            "outlives-distinct-wide width={width} work={} bytes={}",
            paid.get().0,
            paid.get().1
        );
        assert!(
            paid.get().0 < width * 800 + 100,
            "{width}: {}",
            paid.get().0
        );
        assert!(
            paid.get().1 < width * 10000 + 1000,
            "{width}: {}",
            paid.get().1
        );
    }
    Ok(())
}

#[test]
fn demanded_unknown_and_quantified_inputs_keep_explicit_boundaries_and_hot_widths() -> TestResult {
    use crate::types::{
        RegionId, TypeId,
        parameters::{clauses::Quantified, predicates::Predicate},
    };
    let report = crate::test_source::resolve("struct Value<'a>{value:&'a u8}", true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let root = item(&report, "Value")?;
    let mut values = Bounds::default();
    assert!(append(&mut values, &mut facts, ItemId(u32::MAX), &|_, _| Ok(())).is_err());
    assert!(values.items.is_empty());
    let clause = Clause::ForAll(facts.types.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: Predicate {
            ty: TypeId::UNIT,
            bound: Bound::Outlives(RegionId::bound(0, 0)?),
        },
    })?);
    let error = values
        .append_reached(
            &mut facts.types,
            &facts.aggregates,
            root,
            &HashMap::default(),
            |_| [clause],
            &|_, _| Ok(()),
        )
        .err()
        .ok_or("quantified boundary")?;
    assert!(error.contains("quantified"));
    assert!(values.items.is_empty());
    println!(
        "outlives-layout Budget={} Scan={} Bounds={} QueryCache={} TypeComponents={} Kind={} Body={} Node={}",
        size_of::<super::super::super::Budget<'_>>(),
        size_of::<super::super::super::Scan<'_>>(),
        size_of::<Bounds>(),
        size_of::<crate::types::outlives::QueryCache>(),
        size_of::<crate::types::outlives::TypeComponents>(),
        size_of::<crate::types::Kind>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::syntax::ast::Node>()
    );
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    Ok(())
}
