//! Reached variance shares completed dependencies without importing caller proof.
use super::*;
use crate::{resolve::model::DefKind, sema::model::Declarations};
use std::{
    cell::{Cell, RefCell},
    fmt::Write,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type PreviousSolver = (
    Vec<Variance>,
    crate::support::hash::HashMap<(Term, Term), Term>,
    Vec<usize>,
    Vec<Term>,
    usize,
);

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

fn refused_attempt(
    facts: &Declarations,
    owner: ItemId,
    expected: &Variances,
    report: &crate::resolve::expansion::Report,
    failure: usize,
) -> TestResult {
    let mut attempted = Variances::default();
    let calls = Cell::new(0_usize);
    let work = Cell::new(0_usize);
    let bytes = Cell::new(0_usize);
    let denied = Cell::new(false);
    let pay = |steps, storage| {
        let index = calls.get();
        calls.set(index + 1);
        work.set(work.get().saturating_add(steps));
        bytes.set(bytes.get().saturating_add(storage));
        if denied.get() || index == failure {
            denied.set(true);
            Err("test sticky variance payment refused")
        } else {
            Ok(())
        }
    };
    assert!(
        append(&mut attempted, facts, owner, &pay).is_err(),
        "failure {failure}"
    );
    assert!(attempted.ranges.is_empty() && attempted.values.is_empty());
    assert_eq!(attempted.steps, 0);
    let first = (work.get(), bytes.get());
    assert!(append(&mut attempted, facts, owner, &pay).is_err());
    assert_eq!((work.get(), bytes.get()), (first.0 + 1, first.1));
    append(&mut attempted, facts, owner, &|_, _| Ok(()))?;
    for name in ["A", "B"] {
        assert_eq!(
            attempted.get(item(report, name)?),
            expected.get(item(report, name)?)
        );
    }
    Ok(())
}

#[test]
fn every_scratch_growth_is_paid_before_publication_and_failed_attempts_can_resume() -> TestResult {
    let report = crate::test_source::resolve(
        "struct A<'a>{f:fn(B<'a>)}struct B<'a>{next:*const A<'a>,r:&'a u8}",
        true,
    )?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let owner = item(&report, "A")?;
    let trace = RefCell::new(Vec::new());
    let mut expected = Variances::default();
    append(&mut expected, &facts, owner, &|work, bytes| {
        trace.borrow_mut().push((work, bytes));
        Ok(())
    })?;
    assert_eq!(expected.get(owner), facts.lifetimes.variances.get(owner));
    assert!(
        trace
            .borrow()
            .iter()
            .filter(|(_, bytes)| *bytes != 0)
            .count()
            >= 12
    );
    for failure in 0..trace.borrow().len() {
        refused_attempt(&facts, owner, &expected, &report, failure)?;
    }
    assert_eq!(
        size_of::<super::super::Solver<'_>>(),
        size_of::<PreviousSolver>() + size_of::<Option<Payment<'_>>>()
    );
    println!(
        "variance Graph={} Solver={} previousSolver={} Variances={}",
        size_of::<Graph<'_>>(),
        size_of::<super::super::Solver<'_>>(),
        size_of::<PreviousSolver>(),
        size_of::<Variances>()
    );
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn zero_scratch_storage_and_small_work_budgets_fail_before_cached_rows_change() -> TestResult {
    let report = crate::test_source::resolve(
        "struct Leaf<'a>{r:&'a u8}struct Wrap<'a>{leaf:Leaf<'a>}",
        true,
    )?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let leaf = item(&report, "Leaf")?;
    let wrapper = item(&report, "Wrap")?;
    for (work_limit, byte_limit) in [
        (usize::MAX, 0),
        (0, usize::MAX),
        (8, usize::MAX),
        (usize::MAX, 128),
    ] {
        let mut values = Variances::default();
        append(&mut values, &facts, leaf, &|_, _| Ok(()))?;
        let before = (values.steps, values.ranges.clone(), values.values.clone());
        let attempted = Cell::new((0_usize, 0_usize));
        let pay = |work, bytes| {
            let old = attempted.get();
            let next = (old.0.saturating_add(work), old.1.saturating_add(bytes));
            attempted.set(next);
            if next.0 > work_limit || next.1 > byte_limit {
                Err("test variance budget exceeded")
            } else {
                Ok(())
            }
        };
        assert!(append(&mut values, &facts, wrapper, &pay).is_err());
        assert_eq!(
            (values.steps, &values.ranges, &values.values),
            (before.0, &before.1, &before.2)
        );
        assert!(attempted.get().0 > work_limit || attempted.get().1 > byte_limit);
        append(&mut values, &facts, wrapper, &|_, _| Ok(()))?;
        assert_eq!(values.get(wrapper), facts.lifetimes.variances.get(wrapper));
    }
    Ok(())
}
