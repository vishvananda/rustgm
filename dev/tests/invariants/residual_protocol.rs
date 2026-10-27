//! Frozen source protocol calls retain lexical, ownership and publication proofs.
use super::*;
use crate::{
    pipeline::{self, Analysis},
    resolve::{expansion::Report, model::DefId},
    sema::compiler_items::try_protocol,
    syntax::ast::{Kind as Syntax, NodeId},
};
#[path = "residual_evidence.rs"]
mod evidence;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-try-blocks/{name}.rs"
    ))?)
}
fn analyze(source: &str) -> Result<(Report, Analysis)> {
    let report = crate::test_source::source_provider(source, "try_protocol")?;
    let analysis = match pipeline::analyze_source(&report) {
        Ok(analysis) => analysis,
        Err(error) => {
            if let Err(detail) = evidence::details(&report) {
                eprintln!("residual evidence: {detail}");
            }
            return Err(error.into());
        }
    };
    Ok((report, analysis))
}
fn root(report: &Report, facts: &Declarations, name: &str) -> Result<DefId> {
    facts
        .functions
        .iter()
        .find_map(|function| {
            let actual = &report.resolution.definitions[function.definition.0 as usize];
            (report.package.names.get(actual.name) == name).then_some(function.definition)
        })
        .ok_or_else(|| format!("missing real function {name}").into())
}
fn lower(report: &Report, mut analysis: Analysis, names: &[&str]) -> Result {
    let roots = names
        .iter()
        .map(|name| root(report, &analysis.facts, name))
        .collect::<Result<Vec<_>>>()?;
    crate::lower::instances::build_roots_prepared(
        report,
        &mut analysis.facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        analysis.prepared,
    )?;
    Ok(())
}
#[test]
fn genuine_protocol_and_optional_try_tag_keep_all_source_call_bodies() -> Result {
    let ordinary = crate::test_source::fixture(
        "pa21/tests/spec/contracts/polymorphic-outputs/accepted/semantics/result-supertrait-equality.rs",
    )?;
    let report = crate::test_source::resolve(&ordinary, true)?;
    pipeline::analyze_source(&report)?;
    let missing = ordinary.replace("Product<Residual = Self>", "Product");
    assert_ne!(ordinary, missing);
    let report = crate::test_source::resolve(&missing, true)?;
    assert!(pipeline::analyze_source(&report).is_err());
    for name in ["source-protocol", "source-without-try-role"] {
        let (report, analysis) = analyze(&fixture(name)?)?;
        let contexts = analysis
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.call_contexts.as_deref())
            .ok_or("actual call owner")?;
        assert!(
            contexts
                .plans
                .iter()
                .any(|plan| matches!(plan.kind, Kind::Block { .. }))
        );
        assert!(
            contexts
                .plans
                .iter()
                .any(|plan| matches!(plan.kind, Kind::Branch { target: None, .. }))
        );
        assert!(contexts.plans.iter().any(|plan| matches!(
            plan.kind,
            Kind::Branch {
                target: Some(_),
                ..
            }
        )));
        for plan in &contexts.plans {
            let owner = plan.source_use.owner(&analysis.facts)?;
            let Some(Operation::Try { plan: id }) = analysis
                .facts
                .node(plan.node)
                .and_then(|node| node.operation)
            else {
                return Err("actual plan publication".into());
            };
            assert!(id.current(&analysis.facts, owner, plan.node)? == *plan);
            assert!(!analysis.facts.methods.calls.contains_key(&plan.node));
        }
        for role in [
            crate::support::bindings::LangItem::ControlFlowContinue,
            crate::support::bindings::LangItem::ControlFlowBreak,
        ] {
            let claimed = report
                .resolution
                .source_bindings
                .as_deref()
                .and_then(|reserved| reserved.language(role))
                .ok_or("actual variant role")?;
            let actual = &report.resolution.definitions[claimed.definition.0 as usize];
            assert_eq!(actual.namespace, crate::resolve::model::Namespace::Value);
            assert_eq!(actual.kind, crate::resolve::model::DefKind::Variant);
        }
        lower(
            &report,
            analysis,
            &[
                "try_tail",
                "try_function",
                "try_local",
                "try_nested",
                "try_loop",
                "try_custom",
                "try_borrow",
            ],
        )?;
    }
    Ok(())
}
#[test]
fn actual_region_move_and_loop_drop_consumers_keep_residual_exits() -> Result {
    let (report, analysis) = analyze(&fixture("source-drop-exit")?)?;
    lower(&report, analysis, &["try_drop"])?;
    for (name, root_name, category) in [
        ("source-borrow-escape", "leak", "lifetime"),
        ("source-moved-output", "moved", "move"),
    ] {
        let (report, analysis) = analyze(&fixture(name)?)?;
        let error = lower(&report, analysis, &[root_name])
            .err()
            .ok_or_else(|| format!("{name} escaped its actual body proof"))?;
        assert!(error.to_string().contains(category), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn source_closure_boundaries_unused_types_and_wrong_protocol_owners_reject() -> Result {
    for name in ["source-closure-boundary", "source-unused-type-error"] {
        let source = fixture(name)?;
        let report = crate::test_source::source_provider(&source, "try_protocol")?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or_else(|| format!("{name} gained proof"))?;
        assert!(
            !error.message.contains("not implemented"),
            "{name}: {error}"
        );
        assert!(
            !error.message.contains("source Try requires"),
            "{name}: {error}"
        );
    }
    let source = fixture("source-protocol")?;
    for changed in [
        source.replace("#[lang=\"branch\"]", ""),
        source
            .replace(
                "#[lang=\"branch\"]fn branch",
                "#[lang=\"branch\"]fn changed",
            )
            .replace(
                "pub trait FromResidual",
                "#[lang=\"Try\"]pub trait FromResidual",
            ),
    ] {
        assert!(
            analyze(&changed).is_err(),
            "wrong/missing tagged method gained authority"
        );
    }
    Ok(())
}
#[test]
fn current_plan_rejects_other_bodies_nodes_and_changed_source_images() -> Result {
    let (mut report, analysis) = analyze(&fixture("source-protocol")?)?;
    let contexts = analysis
        .facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.call_contexts.as_deref())
        .ok_or("current call owner")?;
    let plan = *contexts
        .plans
        .iter()
        .find(|plan| matches!(plan.kind, Kind::Block { .. }))
        .ok_or("actual Try block row")?;
    let Some(Operation::Try { plan: id }) = analysis
        .facts
        .node(plan.node)
        .and_then(|node| node.operation)
    else {
        return Err("actual Try operation".into());
    };
    let owner = plan.source_use.owner(&analysis.facts)?;
    let wrong = NodeKey {
        node: NodeId::new(u32::MAX),
        ..owner
    };
    assert!(id.current(&analysis.facts, wrong, plan.node).is_err());
    assert!(id.current(&analysis.facts, owner, wrong).is_err());
    let original = report.package.units[0].configured.ast.nodes[plan.node.node.get() as usize].kind;
    let Syntax::TryBlock { body, .. } = original else {
        return Err("actual Try block wrapper".into());
    };
    report.package.units[0].configured.ast.nodes[plan.node.node.get() as usize].kind =
        Syntax::TryBlock {
            body,
            gate: u32::MAX,
        };
    assert!(
        crate::sema::try_plans::checked(&report, &analysis.facts, owner, plan.node, id).is_err()
    );
    report.package.units[0].configured.ast.nodes[plan.node.node.get() as usize].kind = original;
    report.package.units[0].source = report.package.units[0].source.physical_copy();
    assert!(try_protocol::source(&report, &analysis.facts, plan.node).is_err());
    Ok(())
}
#[test]
fn retained_try_storage_and_empty_paths_keep_actual_enclosing_layouts() -> Result {
    assert_eq!(size_of::<Operation>(), 16);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<SourceCallUse>(), 4);
    assert_eq!(size_of::<TryPlanId>(), 4);
    let full = fixture("source-protocol")?;
    let prefix = full
        .split_once("#[unsafe(no_mangle)]")
        .ok_or("actual source foundation")?
        .0;
    for width in [8, 32, 128] {
        use std::fmt::Write;
        let mut source = prefix.to_owned();
        for index in 0..width {
            write!(source, "fn p{index}()->Outcome<u8>{{try{{7}}}}")?;
        }
        let (_, analysis) = analyze(&source)?;
        let contexts = analysis
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.call_contexts.as_deref())
            .ok_or("actual current context")?;
        assert_eq!(contexts.plans.len(), width);
        println!(
            "Try width={width} rows={} rowcap={} plans={} capacity={} planbytes={} Contexts={} CallTypes={} Frame={} State={} Operation={} NodeFacts={} Catalog={} steps={}",
            contexts.rows.len(),
            contexts.rows.capacity(),
            contexts.plans.len(),
            contexts.plans.capacity(),
            contexts.plans.capacity() * size_of::<Plan>(),
            size_of::<CallContexts>(),
            size_of::<crate::sema::bodies::calls::CallTypes>(),
            size_of::<crate::sema::bodies::Frame>(),
            size_of::<crate::sema::bodies::State>(),
            size_of::<Operation>(),
            size_of::<crate::sema::model::NodeFacts>(),
            size_of::<crate::sema::compiler_items::Catalog>(),
            analysis.facts.body_work.inference_steps
        );
    }
    let plain = prefix
        .split_once("pub enum ControlFlow")
        .ok_or("actual ordinary declarations")?
        .0
        .to_owned()
        + "pub fn plain()->u8{7}";
    let (_, analysis) = analyze(&plain)?;
    assert!(
        analysis
            .facts
            .source_bindings
            .as_deref()
            .is_some_and(|catalog| catalog.call_contexts.is_none())
    );
    Ok(())
}
#[test]
fn retained_rows_fail_before_growth_and_preserve_prior_current_context() -> Result {
    crate::sema::traits::results::consequence_controls()?;
    let (report, mut analysis) = analyze(&fixture("source-protocol")?)?;
    let site = NodeKey {
        file: crate::resolve::unit::FileId(0),
        node: report.package.units[0].configured.ast.root,
    };
    try_protocol::source(&report, &analysis.facts, site)?;
    let contexts = analysis
        .facts
        .source_bindings
        .as_deref_mut()
        .and_then(|catalog| catalog.call_contexts.as_deref_mut())
        .ok_or("actual current rows")?;
    let before = (
        contexts.rows.len(),
        contexts.rows.capacity(),
        contexts.plans.len(),
        contexts.plans.capacity(),
    );
    for _ in 0..2 {
        assert!(contexts.reserve_try(MAX_ROWS + 1).is_err());
    }
    assert_eq!(
        before,
        (
            contexts.rows.len(),
            contexts.rows.capacity(),
            contexts.plans.len(),
            contexts.plans.capacity()
        )
    );
    let before = crate::sema::header_attributes::source_contract_storage(&analysis.facts)
        .ok_or("actual ledger")?
        .0;
    assert!(
        crate::sema::header_attributes::charge_source_retention(
            &analysis.facts,
            crate::support::crates::MAX_COMPONENTS,
            0
        )
        .is_err()
    );
    let failed = crate::sema::header_attributes::source_contract_storage(&analysis.facts)
        .ok_or("actual failed ledger")?
        .0;
    assert!(failed > before);
    for _ in 0..2 {
        assert!(try_protocol::source(&report, &analysis.facts, site).is_err());
    }
    let retried = crate::sema::header_attributes::source_contract_storage(&analysis.facts)
        .ok_or("actual retry ledger")?
        .0;
    assert!(retried > failed);
    Ok(())
}
