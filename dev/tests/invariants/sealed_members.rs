//! Final trait bodies keep source authority and direct dispatch, without a slot.
use crate::{
    pipeline,
    resolve::{
        model::{DefId, DefKind},
        unit::{FileId, Unit},
    },
    syntax::{ast::Kind as NodeKind, gate_sites},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/final-methods/{name}.rs"
    ))?)
}
fn report(name: &str) -> Result<crate::resolve::expansion::Report> {
    Ok(crate::test_source::resolve(&source(name)?, true)?)
}
fn definition(report: &crate::resolve::expansion::Report, name: &str) -> Result<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::Function && report.package.names.get(d.name) == name)
        .ok_or("actual function")?;
    Ok(DefId(u32::try_from(index)?))
}
#[test]
fn final_and_ordinary_provided_calls_keep_actual_bodies_and_owners() -> Result {
    for name in ["provided-call", "ordinary-override"] {
        let report = report(name)?;
        let analysis = pipeline::analyze_source(&report)?;
        let method = definition(&report, "value")?;
        assert_eq!(
            super::is_final_trait_method(&report, &analysis.facts, method)?,
            name == "provided-call"
        );
        let functions = &analysis.facts.functions;
        let actual = functions
            .iter()
            .find(|f| f.definition == method)
            .ok_or("actual trait body")?;
        assert!(actual.body.is_some());
    }
    let report = crate::test_source::source_provider(&source("source-share")?, "final_share")?;
    let analysis = pipeline::analyze_source(&report)?;
    let method = definition(&report, "share")?;
    assert!(super::is_final_trait_method(
        &report,
        &analysis.facts,
        method
    )?);
    let contracts = crate::body::regions::collect(&analysis.facts)?;
    crate::borrow::regions::check(
        &analysis.facts.types,
        &analysis.facts.lifetimes.variances,
        &contracts,
    )?;
    Ok(())
}
#[test]
fn final_overrides_wrong_owners_missing_bodies_and_unused_bad_bodies_reject() -> Result {
    for (name, message) in [
        ("final-override", "cannot override final trait method"),
        ("free-owner", "actual trait owner and defining body"),
        ("inherent-owner", "actual trait owner and defining body"),
        ("impl-owner", "actual trait owner and defining body"),
        ("body-required", "actual trait owner and defining body"),
        ("unused-invalid-body", "type"),
    ] {
        let report = report(name)?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or_else(|| format!("{name} gained invalid final semantics"))?;
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn final_gate_uses_exact_keyword_even_when_configured_out() -> Result {
    report("literal-allowance")?;
    for name in [
        "missing-feature",
        "default-is-distinct",
        "forwarded-allowance",
        "disabled-missing-feature",
    ] {
        let error = report(name)
            .err()
            .ok_or_else(|| format!("{name} gained feature authority"))?;
        assert!(
            error.to_string().contains("final_associated_functions"),
            "{name}: {error}"
        );
    }
    let disabled = source("disabled-missing-feature")?.replace(
        "#![allow",
        "#![feature(final_associated_functions)]\n#![allow",
    );
    let disabled = crate::test_source::resolve(&disabled, true)?;
    assert!(
        !disabled
            .resolution
            .definitions
            .iter()
            .any(|d| d.kind == DefKind::Trait && disabled.package.names.get(d.name) == "T")
    );
    assert!(disabled.pre_expansion_warnings()?.is_empty());
    Ok(())
}
#[test]
fn final_dynamic_calls_are_static_while_ordinary_methods_retain_slots() -> Result {
    for name in ["dyn-final", "dyn-ordinary", "dyn-generic-final"] {
        let report = report(name)?;
        let mut analysis = pipeline::analyze_source(&report)?;
        let entry = definition(
            &report,
            if name == "dyn-generic-final" {
                "call"
            } else {
                "entry"
            },
        )?;
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut analysis.facts,
            crate::lower::Roots {
                entry: None,
                exports: &[entry],
            },
            analysis.prepared,
        )?;
        assert!(!program.symbols.is_empty());
        let method = definition(&report, "value")?;
        let owner = analysis
            .facts
            .associated
            .trait_owner(method)
            .ok_or("actual trait owner")?;
        let mut queries = crate::sema::dyn_compat::Queries::default();
        let contract = queries.demand(&report, &mut analysis.facts, owner)?;
        assert_eq!(contract.methods.contains(&method), name == "dyn-ordinary");
        let mut dynamic_instances = 0;
        let mut virtual_instances = 0;
        let callables: Vec<_> = analysis
            .facts
            .methods
            .calls
            .values()
            .map(|call| call.callable)
            .collect();
        for ty in callables {
            if crate::sema::dispatch::object_method(&mut analysis.facts, ty)?
                .is_some_and(|(actual, _)| actual == method)
            {
                dynamic_instances += 1;
                let instance = crate::types::instance::Instance::new(&analysis.facts.types, ty)?;
                virtual_instances += usize::from(crate::sema::vtable_plans::prepare_call(
                    &report,
                    &mut analysis.facts,
                    instance,
                )?);
            }
        }
        assert!(
            dynamic_instances > 0,
            "{name} did not reach actual dynamic Self"
        );
        if name == "dyn-ordinary" {
            assert!(virtual_instances > 0, "ordinary call lost its virtual plan");
        } else {
            assert_eq!(virtual_instances, 0, "final method gained a virtual slot");
        }
    }
    let report = report("dyn-generic-ordinary")?;
    assert!(pipeline::analyze_source(&report).is_err());
    Ok(())
}
#[test]
fn final_policy_is_not_defaultness_or_transportable_absence() -> Result {
    let mut report = report("provided-call")?;
    let analysis = pipeline::analyze_source(&report)?;
    let method = definition(&report, "value")?;
    let key = report.resolution.definitions[method.0 as usize]
        .node
        .ok_or("actual final source")?;
    assert!(report.active_defaultness(key)?.is_none());
    let modifier = report
        .active_finalness(key)?
        .ok_or("actual final modifier")?;
    let error = crate::sema::crates::authored_export::require_defaultness_transport(&report, key)
        .err()
        .ok_or("final policy escaped")?;
    assert_eq!(
        error.message,
        "source final trait method contract transport is not implemented"
    );
    assert_eq!(error.span.start, modifier.span.start);
    report.resolution.definitions[method.0 as usize].node = None;
    let missing = super::is_final_trait_method(&report, &analysis.facts, method)
        .err()
        .ok_or("missing local source became a final policy absence")?;
    assert_eq!(
        missing.message,
        "imported or missing final trait method policy is not available"
    );
    report.resolution.definitions[method.0 as usize].node = Some(key);
    report.package.units[key.file.0 as usize].source =
        report.package.unit(key.file).source.physical_copy();
    assert!(report.active_finalness(key).is_err());
    Ok(())
}
#[test]
fn sparse_final_rows_transfer_replay_and_keep_ordinary_layouts() -> Result {
    use std::fmt::Write;
    for width in [8, 32, 128] {
        let mut input = String::from("#![feature(final_associated_functions)]trait T{");
        for index in 0..width {
            write!(input, "final fn f{index}(&self)->u8{{7}}")?;
        }
        input.push('}');
        let report = crate::test_source::resolve(&input, true)?;
        crate::sema::declarations::analyze(&report)?;
        let sites = report
            .package
            .unit(FileId(0))
            .configured
            .ast
            .gate_sites
            .as_ref()
            .ok_or("actual sparse final owner")?;
        assert_eq!(sites.entries().len(), width);
        assert!(
            !report
                .package
                .unit(FileId(0))
                .configured
                .ast
                .has_defaultness()
        );
        println!("Final width={width} sparse={:?}", sites.storage());
    }
    let mut names = crate::support::names::Names::default();
    let unit = Unit::parse(
        crate::support::source::SourceFile::new(source("provided-call")?.into_bytes())?,
        &crate::expand::config::Config::default(),
        &mut names,
    )
    .map_err(|e| e.message)?;
    for replay in [
        unit.fresh_instance().map_err(|e| e.message)?,
        unit.fresh_instance().map_err(|e| e.message)?,
    ] {
        replay
            .configured
            .ast
            .validate_gate_sites_source(replay.source.original_image())
            .map_err(|e| e.message)?;
        let rows = replay
            .configured
            .ast
            .gate_sites
            .as_ref()
            .ok_or("replayed owner")?;
        assert_eq!(rows.entries().len(), 1);
        assert_eq!(rows.entries()[0].kind, gate_sites::Kind::FinalFunction);
        assert!(matches!(
            replay.configured.ast.node(rows.entries()[0].node).kind,
            NodeKind::Item(_)
        ));
    }
    let ordinary = crate::test_source::resolve("fn ordinary(){}", true)?;
    assert!(
        ordinary
            .package
            .unit(FileId(0))
            .configured
            .ast
            .gate_sites
            .is_none()
    );
    assert_eq!(size_of::<gate_sites::Declaration>(), 24);
    assert_eq!(size_of::<gate_sites::Declarations>(), 112);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    println!(
        "Final layouts Item={} Function={} Syntax={} Unit={} Definition={} Scope={} Report={}",
        size_of::<crate::syntax::item::Item>(),
        size_of::<crate::sema::model::Function>(),
        size_of::<crate::syntax::ast::Syntax<crate::literal::storage::StoredValue>>(),
        size_of::<Unit>(),
        size_of::<crate::resolve::model::Definition>(),
        size_of::<crate::resolve::model::Scope>(),
        size_of::<crate::resolve::expansion::Report>()
    );
    Ok(())
}
