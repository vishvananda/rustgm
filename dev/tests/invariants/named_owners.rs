//! Actual declaration queries, not synthetic proof rows, own these scopes.
use super::*;
use crate::{sema::declarations, support::hash::HashSet};
use std::cell::RefCell;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct Visit {
    entered: bool,
    scope: Option<ValueScope>,
    context: u32,
    pending: usize,
}
thread_local! {
    static TRACE: RefCell<Option<Vec<Visit>>> = const { RefCell::new(None) };
}
struct Trace;
impl Trace {
    fn begin() -> TestResult<Self> {
        TRACE.with(|trace| -> TestResult {
            let mut trace = trace.try_borrow_mut()?;
            if trace.is_some() {
                return Err("named owner trace cannot reenter".into());
            }
            *trace = Some(Vec::new());
            Ok(())
        })?;
        Ok(Self)
    }
    fn finish(self) -> TestResult<Vec<Visit>> {
        let visits = TRACE
            .with(|trace| trace.borrow_mut().take())
            .ok_or("trace")?;
        drop(self);
        Ok(visits)
    }
}
impl Drop for Trace {
    fn drop(&mut self) {
        TRACE.with(|trace| {
            drop(trace.replace(None));
        });
    }
}
pub(super) fn record(entered: bool, scope: Option<ValueScope>, context: u32, pending: usize) {
    TRACE.with(|trace| {
        if let Some(visits) = trace.borrow_mut().as_mut() {
            visits.push(Visit {
                entered,
                scope,
                context,
                pending,
            });
        }
    });
}
fn index(queries: &Queries<'_>, name: &str) -> TestResult<usize> {
    queries
        .typing
        .facts
        .initializers
        .iter()
        .position(|initializer| {
            queries
                .report
                .package
                .names
                .get(queries.report.resolution.definitions[initializer.definition.0 as usize].name)
                == name
        })
        .ok_or_else(|| format!("actual named initializer {name}").into())
}
fn request(queries: &mut Queries<'_>, index: usize) -> TestResult {
    queries
        .work
        .push(Work::Enter(super::super::Query::Constant(index)));
    queries.drain()?;
    Ok(())
}

#[test]
fn actual_nested_named_queries_restore_outer_scope_and_warm_queries_add_no_event() -> TestResult {
    let report = crate::test_source::resolve("const A:u8=B;const B:u8=7;const C:u8=9;", true)?;
    let mut facts = declarations::analyze(&report)?;
    let mut queries = Queries::new(&report, &mut facts);
    let a = index(&queries, "A")?;
    let b = index(&queries, "B")?;
    let c = index(&queries, "C")?;
    let trace = Trace::begin()?;
    request(&mut queries, a)?;
    let visits = trace.finish()?;
    let scope = |i: usize| {
        Some(ValueScope::Initializer {
            body: queries.typing.facts.initializers[i].semantic_source(),
            index: i,
        })
    };
    assert_eq!(
        visits
            .iter()
            .map(|v| (v.entered, v.scope))
            .collect::<Vec<_>>(),
        [
            (true, scope(a)),
            (true, scope(b)),
            (false, scope(a)),
            (false, None)
        ]
    );
    assert!(visits.iter().all(|v| v.context == 0));
    assert_eq!(queries.value_scope, None);
    assert!(
        queries.pattern_frames.is_none(),
        "ordinary queries allocate no pattern frame"
    );
    let capacity = queries.work.observations();
    let before = queries.steps;
    let trace = Trace::begin()?;
    request(&mut queries, a)?;
    assert!(trace.finish()?.is_empty());
    assert_eq!(
        queries.steps - before,
        1,
        "only the existing ready-query dispatch runs"
    );
    assert_eq!(queries.work.observations(), capacity);
    queries.typing.initializer(c)?;
    let initializer = queries.typing.facts.initializers[c];
    let previous = queries
        .typing
        .facts
        .constant_values
        .get(&initializer.definition)
        .copied();
    let trace = Trace::begin()?;
    queries.prepare(initializer.value)?;
    queries
        .work
        .push(Work::Enter(super::super::Query::Node(initializer.value)));
    queries.drain()?;
    assert!(trace.finish()?.is_empty());
    assert!(
        queries
            .typing
            .facts
            .constant_values
            .get(&initializer.definition)
            .copied()
            == previous
    );
    assert_eq!(
        queries.value_scope, None,
        "a sibling anonymous request inherits no named permission"
    );
    Ok(())
}

#[test]
fn named_dependency_suspension_keeps_current_owner_and_queued_restore() -> TestResult {
    let report = crate::test_source::resolve("const A:u8=B;const B:u8=7;", true)?;
    let mut facts = declarations::analyze(&report)?;
    let mut queries = Queries::new(&report, &mut facts);
    let a = index(&queries, "A")?;
    let b = index(&queries, "B")?;
    let mut ready = HashSet::from_iter([queries.typing.facts.initializers[a].definition]);
    queries
        .work
        .push(Work::Enter(super::super::Query::Constant(a)));
    assert!(
        matches!(queries.drain_until(Some(&ready))?, Some(super::super::suspension::Dependency::Initializer(definition)) if definition == queries.typing.facts.initializers[b].definition)
    );
    let owner = queries.value_scope;
    assert!(matches!(owner, Some(ValueScope::Initializer { index, .. }) if index == a));
    let pending = queries.work.observations();
    assert!(pending.0 > 0);
    let (scratch, continuation) = queries.suspend()?;
    let mut resumed = Queries::with_scratch(&report, &mut facts, None, scratch);
    resumed.resume(*continuation);
    assert_eq!(resumed.value_scope, owner);
    assert_eq!(resumed.work.observations(), pending);
    ready.insert(resumed.typing.facts.initializers[b].definition);
    assert!(resumed.drain_until(Some(&ready))?.is_none());
    assert_eq!(resumed.value_scope, None);
    assert_eq!(resumed.work.observations().0, 0);
    assert!(resumed.pattern_frames.is_none());
    println!(
        "Work={} Continuation={} Worklist={}",
        size_of::<Work>(),
        size_of::<super::super::suspension::Continuation>(),
        size_of::<super::super::instances::Worklist>()
    );
    Ok(())
}

#[test]
fn named_cycles_and_failed_restore_budget_abandon_work_without_scope_leaks() -> TestResult {
    let report = crate::test_source::resolve("const A:u8=B;const B:u8=A;const C:u8=7;", true)?;
    let mut facts = declarations::analyze(&report)?;
    let mut queries = Queries::new(&report, &mut facts);
    let a = index(&queries, "A")?;
    let c = index(&queries, "C")?;
    assert!(
        request(&mut queries, a)
            .err()
            .ok_or("cycle accepted")?
            .to_string()
            .contains("cycle")
    );
    assert_eq!(queries.value_scope, None);
    assert_eq!(queries.work.observations().0, 0);
    request(&mut queries, c)?;
    assert_eq!(queries.value_scope, None);
    // Isolate failure after a genuine scope's restore event is popped. No
    // executable permission is asserted by this metadata-only owner control.
    queries.begin_named_value(c)?;
    queries.limit = queries.steps;
    assert!(
        queries
            .drain()
            .err()
            .ok_or("budget bypassed")?
            .to_string()
            .contains("work budget")
    );
    assert!(queries.steps > queries.limit);
    assert_eq!(queries.value_scope, None);
    assert_eq!(queries.work.observations().0, 0);
    Ok(())
}

#[test]
fn named_width_and_depth_charge_linear_stack_work_without_pattern_allocation() -> TestResult {
    use std::fmt::Write;
    for width in [1, 8, 32, 128] {
        for deep in [false, true] {
            let mut source = String::new();
            for i in 0..width {
                write!(&mut source, "const N{i}:u8=")?;
                if deep && i + 1 < width {
                    writeln!(&mut source, "N{};", i + 1)?;
                } else {
                    source.push_str("7;\n");
                }
            }
            if !deep {
                source.push_str("const ROOT:(");
                for _ in 0..width {
                    source.push_str("u8,");
                }
                source.push_str(")=(");
                for i in 0..width {
                    write!(&mut source, "N{i},")?;
                }
                source.push_str(");\n");
            }
            let report = crate::test_source::resolve(&source, true)?;
            let mut facts = declarations::analyze(&report)?;
            let mut queries = Queries::new(&report, &mut facts);
            let root = index(&queries, if deep { "N0" } else { "ROOT" })?;
            let trace = Trace::begin()?;
            request(&mut queries, root)?;
            let visits = trace.finish()?;
            let count = width + usize::from(!deep);
            assert_eq!(visits.len(), count * 2);
            assert!(queries.steps < 100 * count + 1000);
            let peak = visits
                .iter()
                .map(|visit| visit.pending)
                .max()
                .ok_or("trace")?;
            assert!(peak <= 4 * count + 16);
            assert_eq!(queries.work.observations().0, 0);
            assert_eq!(queries.value_scope, None);
            assert!(queries.pattern_frames.is_none());
            assert!(
                queries.typing.facts.constant_executor.is_none(),
                "literal queries require no executable service"
            );
            // Actual constants::Work was already 64 bytes in the published
            // pre-reference compiler (b0b81f5c): Worklist::push copies 4x16
            // bytes; its (Context, Work) vector has an 80-byte stride.
            // The unrelated sema::bodies::Work is 40 bytes.
            assert_eq!(size_of::<Work>(), 64);
            println!(
                "named-width={width} deep={deep} events={} steps={} peak-pending={peak} capacity={}",
                visits.len(),
                queries.steps,
                queries.work.observations().1
            );
        }
    }
    Ok(())
}
