//! Source Always context is distinct from ordinary const-fn and erased pointers.
use super::*;
use crate::{pipeline, support::diagnostics::LintCap, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BASE: &str = "pa31/tests/spec/source-comptime/";
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{BASE}{name}.rs"))?)
}
fn report(source: &str, tag: &str) -> TestResult<Report> {
    let mut config = crate::expand::config::Config::default();
    config.set_lint_cap(LintCap::Allow);
    Ok(test_source::source_provider_config(
        source,
        &tag.replace('-', "_"),
        &config,
    )?)
}
fn function(report: &Report, name: &str) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|row| {
            row.kind == crate::resolve::model::DefKind::Function
                && report.package.names.get(row.name) == name
        })
        .ok_or("actual source function")?;
    Ok(DefId(u32::try_from(index)?))
}
#[test]
fn real_always_bodies_const_roots_aliases_and_erased_pointers_reach_lowir() -> TestResult {
    for name in [
        "const-own",
        "inline-const",
        "item-alias-const",
        "inherent-const",
        "unsafe-const",
        "function-pointer",
        "generic-identity",
        "generic-const-bound",
    ] {
        let source = input(name)?;
        let report = report(&source, name)?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
        let target = match name {
            "inherent-const" => "eval",
            _ => "id",
        };
        let id = function(&report, target)?;
        assert!(facts.is_const_function(id), "{name}");
        assert!(matches!(
            facts.source_comptime_contract(&report, id)?,
            ContractKnowledge::KnownPresent(_)
        ));
        let body = facts.functions[*facts.function_indices.get(&id).ok_or("function index")?]
            .body
            .ok_or("actual body")?;
        assert_eq!(
            facts.source_function_context(&report, id, body)?,
            crate::sema::function_context::FunctionContext::Comptime
        );
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )
        .map_err(|error| format!("{name}: {error}"))?;
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/comptime-v1/candidate")
            .join(name);
        std::fs::create_dir_all(&directory)?;
        std::fs::write(directory.join("source.rs"), source)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(directory.join("provider.lowir"))?,
        )?;
    }
    Ok(())
}
#[test]
fn direct_runtime_and_ordinary_const_fn_calls_do_not_become_const_blocks() -> TestResult {
    for name in [
        "runtime-literal",
        "runtime-argument",
        "runtime-alias",
        "ordinary-const-fn",
    ] {
        let source = input(name)?;
        let report = report(&source, name)?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or("runtime direct call accepted")?;
        assert!(
            error
                .to_string()
                .contains("comptime functions can only be called at compile time"),
            "{name}: {error}"
        );
    }
    for name in [
        "marked-nonconst-callee",
        "unsafe-missing",
        "conditional-bound",
        "nonconst-bound",
    ] {
        let source = input(name)?;
        let outcome = report(&source, name)
            .and_then(|r| pipeline::analyze_source(&r).map(|_| ()).map_err(Into::into));
        assert!(outcome.is_err(), "{name}");
    }
    Ok(())
}
#[test]
fn actual_gate_forms_and_resolved_free_or_inherent_ownership_are_required() -> TestResult {
    for name in [
        "explicit-const",
        "duplicate",
        "list-form",
        "value-form",
        "unsafe-wrapper",
        "trait-method",
        "trait-impl",
        "wrong-owner",
        "foreign-prototype",
    ] {
        let source = input(name)?;
        let outcome = report(&source, name)
            .and_then(|r| pipeline::analyze_source(&r).map(|_| ()).map_err(Into::into));
        assert!(outcome.is_err(), "{name}");
    }
    let source = input("missing-gate")?;
    let outcome = test_source::resolve(&source, true)
        .map_err(Into::into)
        .and_then(|r| pipeline::analyze_source(&r).map(|_| ()).map_err(Into::into));
    let error: Box<dyn std::error::Error> = outcome.err().ok_or("missing gate accepted")?;
    assert!(error.to_string().contains("rustc_attrs"), "{error}");
    let source = input("ordinary-const")?;
    let report = test_source::resolve(&source, true)?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let id = function(&report, "id")?;
    assert!(matches!(
        facts.source_comptime_contract(&report, id)?,
        ContractKnowledge::KnownPresent(_)
    ));
    let source = input("ordinary-runtime")?;
    let report = test_source::resolve(&source, true)?;
    assert!(pipeline::analyze_source(&report).is_err());
    Ok(())
}
#[test]
fn ordinary_macro_closure_and_generic_callers_keep_actual_source_contexts() -> TestResult {
    for name in [
        "ordinary-const",
        "inline-in-closure",
        "constructed-const",
        "macro-allowance",
        "generic-fn-item",
        "fn-pointer-parameter",
        "fn-pointer-call-body",
        "const-api-marker",
    ] {
        let source = input(name)?;
        let report = test_source::resolve(&source, true)?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )
        .map_err(|error| format!("{name}: {error}"))?;
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/comptime-v1/candidate")
            .join(name);
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("source.rs"), source)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(folder.join("provider.lowir"))?,
        )?;
    }
    let source = input("closure-in-const")?;
    let report = test_source::resolve(&source, true)?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("closure inherited enclosing Const permission")?;
    assert!(
        error
            .to_string()
            .contains("comptime functions can only be called at compile time"),
        "{error}"
    );
    Ok(())
}
#[test]
fn const_api_axis_requires_the_actual_always_function_and_stays_distinct() -> TestResult {
    let source = input("const-api-marker")?;
    let report = test_source::resolve(&source, true)?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let id = function(&report, "id")?;
    let inventory = report
        .package
        .library_policy()
        .and_then(crate::resolve::stability::Library::inventory)
        .ok_or("actual API inventory")?;
    let regular = inventory.declaration(id).ok_or("regular API")?;
    let constant = inventory.const_declaration(id).ok_or("const API")?;
    assert_eq!(regular.kind, crate::resolve::stability::PolicyKind::Regular);
    assert_eq!(constant.kind, crate::resolve::stability::PolicyKind::Const);
    assert_ne!(
        regular.feature_name()?.spelling(),
        constant.feature_name()?.spelling()
    );
    assert!(
        facts
            .reject_function_advisory_transport(&report, id)
            .is_err()
    );
    let source = input("const-api-unmarked")?;
    let error = test_source::resolve(&source, true)
        .err()
        .ok_or("unmarked const API accepted")?;
    assert!(
        error.contains("const library stability requires"),
        "{error}"
    );
    Ok(())
}
fn marker_mut(facts: &mut Declarations, key: NodeKey) -> TestResult<&mut Marker> {
    facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|owner| owner.headers.as_deref_mut())
        .and_then(|ledger| ledger.local.declarations.get_mut(&key))
        .and_then(|header| header.promises.as_deref_mut())
        .and_then(|promises| promises.comptime.as_mut())
        .ok_or_else(|| "actual retained marker".into())
}
fn headers_mut(facts: &mut Declarations) -> TestResult<&mut super::super::Headers> {
    Ok(&mut facts
        .must_use_contracts
        .as_deref_mut()
        .ok_or("contracts")?
        .headers
        .as_deref_mut()
        .ok_or("headers")?
        .local)
}
#[test]
fn authentic_marker_queries_reject_foreign_changed_owner_effect_flags_and_exhaustion() -> TestResult
{
    let source = input("const-own")?;
    let report = report(&source, "queries")?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let id = function(&report, "id")?;
    let foreign = self::report(&source, "queries")?;
    assert!(facts.source_comptime_contract(&foreign, id).is_err());
    let ContractKnowledge::KnownPresent(site) = facts.source_comptime_contract(&report, id)? else {
        return Err("marker".into());
    };
    let key = site.declaration;
    let effect = marker_mut(&mut facts, key)?.effect;
    marker_mut(&mut facts, key)?.effect = usize::MAX - 1;
    assert!(facts.source_comptime_contract(&report, id).is_err());
    marker_mut(&mut facts, key)?.effect = effect;
    let slot = marker_mut(&mut facts, key)?.owner_slot;
    marker_mut(&mut facts, key)?.owner_slot = usize::MAX;
    assert!(facts.source_comptime_contract(&report, id).is_err());
    marker_mut(&mut facts, key)?.owner_slot = slot;
    marker_mut(&mut facts, key)?.flags.constant = true;
    assert!(facts.source_comptime_contract(&report, id).is_err());
    marker_mut(&mut facts, key)?.flags.constant = false;
    assert!(facts.source_comptime_contract(&report, id).is_ok());
    let headers = &facts
        .must_use_contracts
        .as_deref()
        .ok_or("contracts")?
        .headers
        .as_deref()
        .ok_or("headers")?
        .local;
    headers.steps.set(crate::support::crates::MAX_COMPONENTS);
    assert!(facts.source_comptime_contract(&report, id).is_err());
    assert!(facts.source_comptime_contract(&report, id).is_err());
    Ok(())
}
#[test]
fn always_contracts_cannot_be_published_as_ordinary_const_metadata() -> TestResult {
    let source = input("const-own")?;
    let report = report(&source, "transport")?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let id = function(&report, "id")?;
    let observed = function(&report, "observed")?;
    assert!(
        facts
            .reject_function_advisory_transport(&report, id)
            .is_err()
    );
    assert!(
        facts
            .reject_function_advisory_transport(&report, observed)
            .is_ok()
    );
    assert!(matches!(
        facts.source_comptime_contract(&report, observed)?,
        ContractKnowledge::KnownAbsent
    ));
    let body = facts.functions[*facts.function_indices.get(&observed).ok_or("index")?]
        .body
        .ok_or("body")?;
    assert!(facts.source_function_context(&report, id, body).is_err());
    Ok(())
}
#[test]
fn pending_or_missing_canonical_witness_is_not_authenticated_absence() -> TestResult {
    let source = input("const-own")?;
    let report = report(&source, "pending")?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let id = function(&report, "id")?;
    let ContractKnowledge::KnownPresent(site) = facts.source_comptime_contract(&report, id)? else {
        return Err("marker".into());
    };
    let key = site.declaration;
    headers_mut(&mut facts)?.comptime_complete = false;
    assert!(!facts.is_comptime_function(id));
    assert!(facts.source_comptime_contract(&report, id).is_err());
    headers_mut(&mut facts)?.comptime_complete = true;
    let definition = headers_mut(&mut facts)?
        .definitions
        .remove(&id)
        .ok_or("binding")?;
    assert!(facts.source_comptime_contract(&report, id).is_err());
    headers_mut(&mut facts)?.definitions.insert(id, definition);
    let promises = headers_mut(&mut facts)?
        .declarations
        .get_mut(&key)
        .ok_or("header")?
        .promises
        .take();
    assert!(!facts.is_comptime_function(id));
    assert!(facts.source_comptime_contract(&report, id).is_err());
    headers_mut(&mut facts)?
        .declarations
        .get_mut(&key)
        .ok_or("header")?
        .promises = promises;
    assert!(facts.source_comptime_contract(&report, id).is_ok());
    Ok(())
}
#[test]
fn cold_promises_share_existing_header_space_and_warm_queries_retain_no_storage() -> TestResult {
    let source = input("const-own")?;
    let report = report(&source, "widths")?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let id = function(&report, "id")?;
    let before = super::super::source_contract_storage(&facts).ok_or("storage")?;
    for _ in 0..128 {
        facts.source_comptime_contract(&report, id)?;
    }
    let after = super::super::source_contract_storage(&facts).ok_or("storage")?;
    assert_eq!(after.1, before.1);
    println!(
        "Header={} Promises={} Headers={} Context={} query128work={}",
        size_of::<Header>(),
        size_of::<super::super::Promises>(),
        size_of::<super::super::Headers>(),
        size_of::<crate::sema::function_context::FunctionContext>(),
        after.0 - before.0
    );
    assert_eq!(size_of::<Header>(), 80);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    let (foundation, _) = source
        .split_once("#[rustc_comptime]")
        .ok_or("fixture foundation")?;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = foundation.to_owned();
        for index in 0..width {
            use std::fmt::Write;
            writeln!(source, "#[rustc_comptime]fn id_{index}(x:u32)->u32{{x}}")?;
        }
        let report = self::report(&source, "growth")?;
        let facts = pipeline::analyze_source(&report)?.facts;
        let storage = super::super::source_contract_storage(&facts).ok_or("growth storage")?;
        println!(
            "Always width={width} work={} bytes={}",
            storage.0, storage.1
        );
        if let Some((work, bytes)) = previous {
            assert!(storage.0 <= work * 5 + 128);
            assert!(storage.1 <= bytes * 5 + 1024);
        }
        previous = Some((storage.0, storage.1));
    }
    Ok(())
}
