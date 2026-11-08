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
    println!(
        "variance seenkey old={} owner_qualified={}",
        size_of::<(TypeId, Term, u16)>(),
        size_of::<(ItemId, TypeId, Term, u16)>()
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

#[test]
fn one_wide_dependency_closure_retains_owner_visits_with_near_linear_work() -> TestResult {
    let mut previous = None;
    for width in [16, 64, 128] {
        let source = shared_bundle(width)?;
        let report = crate::test_source::resolve(&source, true)?;
        let facts = crate::sema::declarations::analyze(&report)?;
        let owner = item(&report, "Bundle")?;
        let paid = Cell::new((0_usize, 0_usize));
        let (mut values, order) = bundle_variances(&facts, owner, &|work, bytes| {
            let before = paid.get();
            paid.set((
                before.0.saturating_add(work),
                before.1.saturating_add(bytes),
            ));
            Ok(())
        })?;
        let leaf = item(&report, "Leaf")?;
        let leaf_position = order
            .iter()
            .position(|&item| item == leaf)
            .ok_or("wide leaf was not reached")?;
        assert!(leaf_position <= 2 && order.len() - leaf_position >= width);
        assert_eq!(values.ranges.len(), width + 2);
        for index in 0..width {
            let wrapper = item(&report, &format!("Wrap{index}"))?;
            assert_eq!(values.get(wrapper), facts.lifetimes.variances.get(wrapper));
        }
        for name in ["Leaf", "Bundle"] {
            let owner = item(&report, name)?;
            assert_eq!(values.get(owner), facts.lifetimes.variances.get(owner));
            assert_eq!(values.get(owner), Some([Variance::Covariant].as_slice()));
        }
        let (work, bytes) = paid.get();
        if let Some((old_width, old_work, old_bytes)) = previous {
            assert!(
                work * old_width < old_work * width * 3 / 2,
                "width={width} work={work} previous={old_work}"
            );
            assert!(
                bytes * old_width < old_bytes * width * 2,
                "width={width} bytes={bytes} previous={old_bytes}"
            );
        }
        previous = Some((width, work, bytes));
        let before = paid.get();
        append(&mut values, &facts, owner, &|work, bytes| {
            let old = paid.get();
            paid.set((old.0 + work, old.1 + bytes));
            Ok(())
        })?;
        assert_eq!(paid.get(), (before.0 + 1, before.1));
        println!("variance oneclosure width={width} work={work} bytes={bytes}");
    }
    Ok(())
}
fn shared_bundle(width: usize) -> TestResult<String> {
    let mut source = String::from("struct Leaf<'a>{");
    for index in 0..width {
        write!(&mut source, "f{index}:[&'a u8;{}],", index + 1)?;
    }
    source.push('}');
    for index in 0..width {
        write!(&mut source, "struct Wrap{index}<'a>{{leaf:Leaf<'a>}}")?;
    }
    source.push_str("struct Bundle<'a>{");
    for index in 0..width {
        write!(&mut source, "w{index}:Wrap{index}<'a>,")?;
    }
    source.push('}');
    Ok(source)
}

fn bundle_variances(
    facts: &Declarations,
    owner: ItemId,
    pay: Payment<'_>,
) -> TestResult<(Variances, Vec<ItemId>)> {
    let mut values = Variances::default();
    let mut order = Vec::new();
    values.append_reached(
        &facts.types,
        &facts.aggregates,
        &facts.definitions,
        owner,
        |owner, ordinal, storage| {
            order.push(owner);
            facts.unreduced.fields(owner, ordinal).unwrap_or(storage)
        },
        pay,
    )?;
    Ok((values, order))
}

/// The scratch capacity a reached-closure graph retains, measured as
/// `retained::vector` and `retained::map` measure owned collections.
fn retained_graph(graph: &Graph<'_>) -> usize {
    let mut allocation = crate::support::retained::Allocation::default();
    allocation.vector(&graph.pending);
    allocation.vector(&graph.result.values);
    allocation.map(&graph.result.ranges);
    allocation.vector(&graph.slots);
    allocation.vector(&graph.variables);
    allocation.vector(&graph.work);
    allocation.set(&graph.seen);
    allocation.add(graph.solver.retained());
    allocation.bytes
}

/// Paid work of the reached variance of `L0<'a> { next: &'a L1<'a> }` through
/// `L{count - 1}<'a> { r: &'a u8 }`, where each field template is published
/// only after the append waits for it. A `restart` append forgets its graph
/// at every wait, so it walks the published prefix again. At every wait the
/// bytes the graph has paid equal the capacity its parked graph retains.
fn waited_chain_work(count: usize, restart: bool) -> TestResult<usize> {
    let mut source = String::new();
    for index in 0..count {
        if index + 1 == count {
            write!(source, "struct L{index}<'a>{{r:&'a u8}}")?;
        } else {
            write!(source, "struct L{index}<'a>{{next:&'a L{}<'a>}}", index + 1)?;
        }
    }
    let report = crate::test_source::resolve(&source, true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let root = item(&report, "L0")?;
    let mut published = crate::types::aggregate::Definitions::default();
    let mut values = Variances::default();
    let mut waiting = PendingReach::new(root);
    let paid = Cell::new(0_usize);
    let graph_bytes = Cell::new(0_usize);
    let pay = |work, bytes| {
        paid.set(paid.get().saturating_add(work));
        graph_bytes.set(graph_bytes.get().saturating_add(bytes));
        Ok(())
    };
    let mut waits = 0;
    while let Some(owner) = values.append_reached_or_pending(
        &facts.types,
        &published,
        &facts.definitions,
        &mut |owner, ordinal, storage| facts.unreduced.fields(owner, ordinal).unwrap_or(storage),
        &pay,
        &mut waiting,
    )? {
        waits += 1;
        let parked = waiting.graph.as_deref().ok_or("parked chain graph")?;
        assert_eq!(
            graph_bytes.get(),
            retained_graph(parked),
            "wait {waits}: paid bytes and parked capacity"
        );
        let definition = facts.aggregates.get(owner).ok_or("chain field template")?;
        published.insert(
            owner,
            definition.kind,
            definition.repr,
            facts.aggregates.variants(definition),
        )?;
        if restart {
            waiting = PendingReach::new(root);
            graph_bytes.set(0);
        }
    }
    assert_eq!(waits, count, "each nominal is waited for once");
    for index in 0..count {
        let owner = item(&report, &format!("L{index}"))?;
        assert_eq!(values.get(owner), facts.lifetimes.variances.get(owner));
    }
    Ok(paid.get())
}

#[test]
fn waited_reached_closures_resume_at_the_owner_they_waited_for() -> TestResult {
    let [small, middle, large] = [64, 256, 512].map(|count| waited_chain_work(count, false));
    let (small, middle, large) = (small?, middle?, large?);
    println!("variance waited chain work {small} {middle} {large}");
    // Near-linear increments: 192 nominals from 64 to 256 and 256 from 256 to
    // 512. Geometric scratch growth pays each relocation once.
    assert!(
        (large - middle) * 192 < (middle - small) * 256 * 3 / 2,
        "resumed chain work {small} {middle} {large} is not linear"
    );
    // The control: restarting at the root after each wait walks the published
    // prefix again, which is quadratic in the chain.
    let restarted = waited_chain_work(512, true)?;
    println!("variance restarted chain work {restarted}");
    assert!(
        restarted > large * 32,
        "restarted chain {restarted} against resumed {large}"
    );
    Ok(())
}

/// Reserve `steps` in turn through the paid vector payment, from empty, and
/// check after each that the bytes charged equal the retained capacity.
fn grow_vector<T>(charged: &Cell<usize>, steps: &[usize], make: impl Fn(usize) -> T) -> TestResult {
    let pay = |_: usize, bytes: usize| {
        charged.set(charged.get().saturating_add(bytes));
        Ok(())
    };
    charged.set(0);
    let mut values = Vec::new();
    let mut next = 0;
    for &step in steps {
        super::super::payments::vector(Some(&pay), &mut values, step)?;
        values.extend((next..next + step).map(&make));
        next += step;
        assert_eq!(
            charged.get(),
            crate::support::retained::vector(&values),
            "vector at {next}"
        );
    }
    Ok(())
}
/// As [`grow_vector`] for a paid table.
fn grow_map<K: Eq + std::hash::Hash, V>(
    charged: &Cell<usize>,
    steps: &[usize],
    make: impl Fn(usize) -> (K, V),
) -> TestResult {
    let pay = |_: usize, bytes: usize| {
        charged.set(charged.get().saturating_add(bytes));
        Ok(())
    };
    charged.set(0);
    let mut values = crate::support::hash::HashMap::default();
    let mut next = 0;
    for &step in steps {
        super::super::payments::map(Some(&pay), &mut values, step)?;
        values.extend((next..next + step).map(&make));
        next += step;
        assert_eq!(
            charged.get(),
            crate::support::retained::map(&values),
            "map at {next}"
        );
    }
    Ok(())
}
fn narrow<T: TryFrom<usize> + Default>(value: usize) -> T {
    T::try_from(value).unwrap_or_default()
}

/// Every paid scratch reservation charges exactly the capacity it adds, for
/// each entry size the reached graph uses, from empty through mixed steps to
/// more than 10,000 entries.
#[test]
fn paid_scratch_reservations_charge_exactly_the_capacity_they_add() -> TestResult {
    let steps: Vec<usize> = [1, 1, 2, 1, 3, 5, 1, 7, 13, 1, 64]
        .into_iter()
        .cycle()
        .take(1200)
        .collect();
    let charged = Cell::new(0_usize);
    grow_vector(&charged, &steps, |index| narrow::<u8>(index % 4))?;
    grow_vector(&charged, &steps, narrow::<u32>)?;
    grow_vector(&charged, &steps, |index| {
        (index, index, narrow::<u16>(index % 7))
    })?;
    grow_vector(&charged, &steps, |index| [index; 6])?;
    grow_map(&charged, &steps, |index| {
        (narrow::<u32>(index), (index, index))
    })?;
    grow_map(&charged, &steps, |index| ((index, index), index))?;
    grow_map(&charged, &steps, |index| {
        (narrow::<u32>(index), narrow::<u32>(index))
    })?;
    let pay = |_: usize, bytes: usize| {
        charged.set(charged.get().saturating_add(bytes));
        Ok(())
    };
    charged.set(0);
    let mut seen = crate::support::hash::HashSet::default();
    for index in 0..6000_usize {
        let key = (
            narrow::<u32>(index),
            narrow::<u32>(index / 2),
            index,
            narrow::<u16>(index % 7),
        );
        assert!(super::super::payments::insert(Some(&pay), &mut seen, key)?);
        assert!(!super::super::payments::insert(Some(&pay), &mut seen, key)?);
        assert_eq!(
            charged.get(),
            seen.capacity() * size_of_val(&key),
            "set at {index}"
        );
    }
    Ok(())
}
