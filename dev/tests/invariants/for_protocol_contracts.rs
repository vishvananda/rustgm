//! Source iteration retains actual method, source and enclosing-body authority.
use super::*;
use crate::{pipeline, resolve::expansion::Report, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-iteration/{name}.rs"
    ))?)
}
fn analyze(source: &str) -> TestResult<(Report, pipeline::Analysis)> {
    let report = test_source::source_provider(source, "source_iteration")?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn lower(source: &str) -> TestResult<crate::lowir::program::Program> {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(source)?;
    Ok(crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn actual_renamed_and_independent_method_parents_retain_source_iteration_plans() -> TestResult {
    for name in [
        "renamed-protocol",
        "missing-iterator-marker",
        "alternate-next-parent",
        "native-protocol",
    ] {
        let (report, analysis) = analyze(&fixture(name)?).map_err(|e| format!("{name}: {e}"))?;
        assert!(analysis.facts.iteration.is_none());
        let contexts = analysis
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.call_contexts.as_deref())
            .ok_or("actual source call context")?;
        let mut count = 0;
        for plan in &contexts.plans {
            if !matches!(plan.kind, Kind::Iteration { .. }) {
                continue;
            }
            let owner = plan.source_use.owner(&analysis.facts)?;
            let Some(Operation::Iteration { plan: id }) =
                analysis.facts.node(plan.node).and_then(|n| n.operation)
            else {
                return Err("actual iteration operation".into());
            };
            assert!(
                crate::sema::protocol_plans::checked(
                    &report,
                    &analysis.facts,
                    owner,
                    plan.node,
                    id
                )? == *plan
            );
            count += 1;
        }
        assert_eq!(count, 1, "{name}");
    }
    Ok(())
}
#[test]
fn actual_protocol_missing_roles_and_required_method_targets_fail() -> TestResult {
    for name in [
        "missing-next",
        "missing-into-iter",
        "provided-next",
        "wrong-next-result",
        "wrong-next-impl",
    ] {
        let error = analyze(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("{name} gained authority"))?;
        assert!(!error.to_string().contains("budget"), "{name}: {error}");
        assert!(
            !error.to_string().contains("not implemented"),
            "{name}: {error}"
        );
    }
    analyze(&fixture("unused-protocol")?)?;
    Ok(())
}
#[test]
fn genuine_iteration_methods_lower_for_the_native_observer() -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-iteration-v1/candidate-native");
    for name in ["native-protocol", "native-drop"] {
        let source = fixture(name)?;
        let program = lower(&source)?;
        let destination = if name == "native-protocol" {
            folder.clone()
        } else {
            folder.join(name)
        };
        std::fs::create_dir_all(&destination)?;
        std::fs::write(destination.join("source.rs"), source)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(destination.join("provider.lowir"))?,
        )?;
    }
    assert_eq!(size_of::<ProtocolPlanId>(), 4);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<Operation>(), 16);
    Ok(())
}

#[test]
fn actual_for_ownership_and_item_lifetimes_survive_desugaring() -> TestResult {
    for (name, category) in [("moved-iterator", "move"), ("escaping-item", "lifetime")] {
        let error = lower(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("{name} escaped its body proof"))?;
        assert!(error.to_string().contains(category), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn current_for_plan_rejects_other_contexts_calls_and_source_images() -> TestResult {
    let (mut report, mut analysis) = analyze(&fixture("renamed-protocol")?)?;
    let contexts = analysis
        .facts
        .source_bindings
        .as_deref()
        .and_then(|c| c.call_contexts.as_deref())
        .ok_or("actual contexts")?;
    let original = *contexts
        .plans
        .iter()
        .find(|p| matches!(p.kind, Kind::Iteration { .. }))
        .ok_or("actual for plan")?;
    let owner = original.source_use.owner(&analysis.facts)?;
    let Some(Operation::Iteration { plan: id }) =
        analysis.facts.node(original.node).and_then(|n| n.operation)
    else {
        return Err("actual for operation".into());
    };
    let wrong = NodeKey {
        node: crate::syntax::ast::NodeId::new(u32::MAX),
        ..owner
    };
    assert!(id.current(&analysis.facts, wrong, original.node).is_err());
    assert!(id.current(&analysis.facts, owner, wrong).is_err());
    let mut changed = original;
    let Kind::Iteration { plan, .. } = &mut changed.kind else {
        return Err("iteration kind".into());
    };
    plan.next.callable = plan.into_iter.callable;
    let contexts = analysis
        .facts
        .source_bindings
        .as_deref_mut()
        .and_then(|c| c.call_contexts.as_deref_mut())
        .ok_or("actual contexts")?;
    let slot = contexts
        .plans
        .iter_mut()
        .find(|p| p.node == original.node)
        .ok_or("actual row")?;
    *slot = changed;
    assert!(
        crate::sema::protocol_plans::checked(&report, &analysis.facts, owner, original.node, id)
            .is_err()
    );
    let contexts = analysis
        .facts
        .source_bindings
        .as_deref_mut()
        .and_then(|c| c.call_contexts.as_deref_mut())
        .ok_or("actual contexts")?;
    *contexts
        .plans
        .iter_mut()
        .find(|p| p.node == original.node)
        .ok_or("actual row")? = original;
    report.package.units[0].source = report.package.units[0].source.physical_copy();
    assert!(
        crate::sema::protocol_plans::checked(&report, &analysis.facts, owner, original.node, id)
            .is_err()
    );
    Ok(())
}

#[test]
fn actual_for_storage_scales_with_loops_and_keeps_ordinary_owners_compact() -> TestResult {
    use std::fmt::Write;
    let full = fixture("renamed-protocol")?;
    let prefix = full
        .split_once("#[unsafe(no_mangle)]")
        .ok_or("actual foundation")?
        .0;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = format!("{prefix}pub fn loops(){{");
        for _ in 0..width {
            write!(source, "for _ in (Values{{pending:true,value:7}}){{}}")?;
        }
        source.push('}');
        let (_, analysis) = analyze(&source)?;
        let contexts = analysis
            .facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.call_contexts.as_deref())
            .ok_or("actual contexts")?;
        assert_eq!(contexts.plans.len(), width);
        assert!(analysis.facts.iteration.is_none());
        let steps = analysis.facts.body_work.inference_steps;
        if let Some(before) = previous {
            assert!(steps <= before * 6, "width={width}: {steps} > {before} * 6");
        }
        previous = Some(steps);
        println!(
            "source-for width={width} steps={steps} plans={} capacity={} planbytes={} OpenPlan={} Plan={} Contexts={} Catalog={} Reservations={}",
            contexts.plans.len(),
            contexts.plans.capacity(),
            contexts.plans.capacity() * size_of::<Plan>(),
            size_of::<crate::sema::bodies::protocol_control::OpenPlan>(),
            size_of::<Plan>(),
            size_of::<CallContexts>(),
            size_of::<crate::sema::compiler_items::Catalog>(),
            size_of::<crate::resolve::bindings::Reservations>()
        );
    }
    assert_eq!(crate::support::bindings::LangItem::ALL.len(), 81);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<Operation>(), 16);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    println!(
        "source-for role banks {:?}",
        crate::sema::compiler_items::Catalog::binding_layouts()
    );
    Ok(())
}
