//! Reached variance shares completed dependencies without importing caller proof.
use super::*;
use crate::{resolve::model::DefKind, sema::model::Declarations};
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
    values: &mut Variances,
    facts: &Declarations,
    item: ItemId,
    pay: Payment<'_>,
) -> TestResult {
    values.append_reached(
        &facts.types,
        &facts.aggregates,
        &facts.definitions,
        item,
        |owner, ordinal, storage| facts.unreduced.fields(owner, ordinal).unwrap_or(storage),
        pay,
    )?;
    Ok(())
}

#[test]
fn reached_nominal_cycles_match_full_inference_and_failed_payment_publishes_nothing() -> TestResult
{
    let report = crate::test_source::resolve(
        "struct Link<'a>{value:&'a u8,next:*const Link<'a>}\
         struct Wrap<'a>{link:Link<'a>}struct Fixed<'a>{value:*mut &'a u8}",
        true,
    )?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let link = item(&report, "Link")?;
    let wrap = item(&report, "Wrap")?;
    let fixed = item(&report, "Fixed")?;
    let mut values = Variances::default();
    let failed = Cell::new(0_usize);
    assert!(
        append(&mut values, &facts, link, &|work, bytes| {
            failed.set(failed.get().saturating_add(work));
            if bytes == 0 {
                Ok(())
            } else {
                Err("test retained payment refused")
            }
        })
        .is_err()
    );
    assert!(failed.get() > 1);
    assert!(values.ranges.is_empty());
    assert!(values.values.is_empty());
    assert_eq!(values.steps, 0);
    let paid = Cell::new((0_usize, 0_usize));
    let pay = |work, bytes| {
        let before = paid.get();
        paid.set((
            before.0.saturating_add(work),
            before.1.saturating_add(bytes),
        ));
        Ok(())
    };
    append(&mut values, &facts, wrap, &pay)?;
    assert_eq!(values.get(link), facts.lifetimes.variances.get(link));
    assert_eq!(values.get(wrap), facts.lifetimes.variances.get(wrap));
    assert!(values.get(fixed).is_none());
    assert_eq!(values.get(link), Some([Variance::Covariant].as_slice()));
    let before = (values.steps, paid.get());
    append(&mut values, &facts, wrap, &pay)?;
    assert_eq!(values.steps, before.0);
    assert_eq!(paid.get(), (before.1.0 + 1, before.1.1));
    append(&mut values, &facts, fixed, &pay)?;
    assert_eq!(values.get(fixed), Some([Variance::Invariant].as_slice()));
    assert_eq!(values.get(fixed), facts.lifetimes.variances.get(fixed));
    Ok(())
}

#[test]
fn shared_wide_dependencies_are_solved_once_and_warm_queries_retain_no_storage() -> TestResult {
    for width in [16, 64, 256] {
        let mut source = String::from("struct Leaf<'a>{");
        for index in 0..width {
            write!(source, "f{index}:&'a u8,")?;
        }
        source.push('}');
        for index in 0..width {
            write!(source, "struct Wrap{index}<'a>{{leaf:Leaf<'a>}}")?;
        }
        let report = crate::test_source::resolve(&source, true)?;
        let facts = crate::sema::declarations::analyze(&report)?;
        let mut values = Variances::default();
        let paid = Cell::new((0_usize, 0_usize));
        let pay = |work, bytes| {
            let before = paid.get();
            paid.set((
                before.0.saturating_add(work),
                before.1.saturating_add(bytes),
            ));
            Ok(())
        };
        for index in 0..width {
            let owner = item(&report, &format!("Wrap{index}"))?;
            append(&mut values, &facts, owner, &pay)?;
            assert_eq!(values.get(owner), facts.lifetimes.variances.get(owner));
        }
        assert_eq!(values.ranges.len(), width + 1);
        assert!(paid.get().0 < 150 * width + 100, "{}", paid.get().0);
        let owner = item(&report, "Wrap0")?;
        let before = paid.get();
        for _ in 0..width {
            append(&mut values, &facts, owner, &pay)?;
        }
        assert_eq!(paid.get(), (before.0 + width, before.1));
    }
    Ok(())
}
