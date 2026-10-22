//! Actual successful and failing child demand must retire the same owner scope.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa25/tests/spec/inline-query-context")
            .join(format!("{name}.rs")),
    )?)
}
#[test]
fn actual_child_success_and_fault_restore_context_and_retire_code() -> Result {
    for name in ["closure", "paid-lifetime", "faulting-child"] {
        let report = crate::test_source::resolve(&fixture(name)?, true)?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        let mut facts = analysis.facts;
        let context = analysis
            .prepared
            .constants
            .clone()
            .ok_or("prepared executor")?;
        let before = {
            let engine = context.engine.borrow();
            (
                engine.roots.len(),
                engine.sessions.len(),
                engine.completed.len(),
                engine.suspended_slots,
            )
        };
        let result = crate::lower::instances::inspect_prepared(
            &report,
            &mut facts,
            analysis.prepared,
            |_, _| (),
        );
        if name == "faulting-child" {
            assert!(
                result
                    .as_ref()
                    .err()
                    .is_some_and(|e| e.to_string().contains("divides by zero")),
                "{result:?}"
            );
        } else {
            result?;
        }
        let mut engine = context.engine.borrow_mut();
        assert!(
            engine
                .original_components
                .as_ref()
                .is_some_and(|rows| rows.active.is_none())
        );
        assert_eq!(
            before,
            (
                engine.roots.len(),
                engine.sessions.len(),
                engine.completed.len(),
                engine.suspended_slots
            )
        );
        let values = facts
            .inline_constants
            .as_deref()
            .ok_or("registry")?
            .instance_values()
            .filter(|(instance, _, _)| {
                matches!(
                    instance.enclosing(),
                    crate::sema::inline_constants::Context::Callable(_)
                )
            })
            .collect::<Vec<_>>();
        if name == "faulting-child" {
            assert!(
                values.is_empty(),
                "failed callable must roll back its ports"
            );
        } else {
            assert!(
                !values.is_empty(),
                "successful materialized values survive retirement"
            );
        }
        for &(instance, result, _) in &values {
            assert!(
                engine
                    .anonymous_root(&report, &mut facts, instance, result)
                    .is_err(),
                "a retained materialized value is not retired execution authority"
            );
        }
        eprintln!(
            "{name}: restored-caches={before:?} callable-values={} code-bytes={}",
            values.len(),
            engine.catalog.bytes
        );
    }
    Ok(())
}
#[test]
fn ordinary_owners_keep_scope_storage_lazy() {
    let engine = Engine::default();
    assert!(engine.original_components.is_none());
    assert_eq!(
        size_of::<crate::borrow::incremental::OwnedParent>(),
        size_of::<usize>()
    );
    assert!(
        size_of::<crate::borrow::incremental::PairedProved>()
            <= size_of::<crate::borrow::incremental::Proved>() + 3 * size_of::<usize>()
    );
    eprintln!(
        "Engine={} Active={} Code={} Proved={} OwnedParent={} PairedProved={} Port={} Pending={}",
        size_of::<Engine>(),
        size_of::<Active>(),
        size_of::<crate::const_eval::code::Code>(),
        size_of::<crate::borrow::incremental::Proved>(),
        size_of::<crate::borrow::incremental::OwnedParent>(),
        size_of::<crate::borrow::incremental::PairedProved>(),
        size_of::<crate::body::conditional_inputs::Port>(),
        size_of::<crate::body::conditional_inputs::Pending>()
    );
}

#[test]
fn actual_port_work_is_bounded_across_reused_original_components() -> Result {
    let mut previous = None;
    let mut slope = None;
    for width in [1, 8, 32, 128] {
        let extra = "let _=const{S::<'a,Tag>::C};".repeat(width - 1);
        let text = fixture("paid-lifetime")?
            .replace("let f=||{const{", &format!("let f=||{{{extra}const{{"));
        let report = crate::test_source::resolve(&text, true)?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        let mut facts = analysis.facts;
        let context = analysis.prepared.constants.clone().ok_or("executor")?;
        crate::lower::instances::inspect_prepared(
            &report,
            &mut facts,
            analysis.prepared,
            |_, _| (),
        )?;
        let engine = context.engine.borrow();
        let count = facts
            .inline_constants
            .as_deref()
            .ok_or("registry")?
            .instance_values()
            .filter(|(instance, _, _)| {
                matches!(
                    instance.enclosing(),
                    crate::sema::inline_constants::Context::Callable(_)
                )
            })
            .count();
        assert_eq!(count, width);
        assert!(
            engine
                .original_components
                .as_ref()
                .is_some_and(|rows| rows.active.is_none())
        );
        assert!(engine.roots.is_empty());
        assert!(engine.sessions.is_empty());
        assert!(engine.completed.is_empty());
        let work = facts.body_work.canonical_steps;
        if let Some((old_width, old_work)) = previous {
            let ports = width - old_width;
            let visits = work - old_work;
            assert_eq!(visits % ports, 0);
            let increment = visits / ports;
            if let Some(expected) = slope {
                assert_eq!(increment, expected);
            }
            slope = Some(increment);
        }
        previous = Some((width, work));
        eprintln!(
            "width={width} actual-callable-values={count} canonical-work={} code-bytes={} anonymous-instances={}",
            facts.body_work.canonical_steps,
            engine.catalog.bytes,
            facts.constant_work.anonymous_instances
        );
    }
    Ok(())
}
