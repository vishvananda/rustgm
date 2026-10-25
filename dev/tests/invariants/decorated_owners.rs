//! Opponents use the actual source-created engine, owner, lookup and retained rows.
use super::*;
use crate::{
    expand::{
        builtin::{Builtin, BuiltinKind},
        config::Config,
    },
    resolve::{bindings::Context, load::InputFile},
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
    syntax::ast::MacroContext,
};
use std::path::Path;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-attribute-builtins/{name}.rs"
    ))?)
}
fn with_engine(
    source: &str,
    provider: bool,
    body: impl FnOnce(&mut Engine<'_, NoFiles>) -> TestResult,
) -> TestResult {
    let config = Config::default();
    let mut files = NoFiles;
    let context = if provider {
        let identity = CrateIdentity::new("minicore", "decorated_owners")?;
        Context {
            crate_name: Some("minicore".into()),
            current: Some(identity.clone()),
            mode: Mode::Source { provider: identity },
            ..Default::default()
        }
    } else {
        Context::default()
    };
    let mut engine = Engine::create(
        InputFile {
            identity: "attributes.rs".into(),
            path: "attributes.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut files,
        &config,
        &context,
    )?;
    body(&mut engine)
}
fn attribute_index(engine: &Engine<'_, NoFiles>) -> TestResult<usize> {
    engine
        .calls
        .iter()
        .position(|call| matches!(call.grammar, CallGrammar::Attribute))
        .ok_or_else(|| "missing actual attribute call".into())
}
fn counts(engine: &Engine<'_, NoFiles>) -> (usize, usize, usize) {
    (
        engine.captures.len(),
        engine.invocations.len(),
        engine.loader.package.marks.context_count(),
    )
}

#[test]
fn actual_attribute_call_keeps_owner_effect_scope_and_failed_warm_budget_identity() -> TestResult {
    with_engine(&source("direct-attribute-empty")?, true, |engine| {
        let error = engine.run().err().ok_or("attribute executed")?;
        assert!(
            error
                .message
                .contains("builtin attribute execution is not implemented")
        );
        let index = attribute_index(engine)?;
        let before = counts(engine);
        let selected = engine.lookup_call(index)?.ok_or("selected actual role")?;
        assert!(matches!(
            engine.definitions[selected].body,
            super::super::Body::Builtin(Builtin::AutodiffForward)
        ));
        validate(
            &engine.loader.package,
            &engine.paths.state.facts,
            &engine.calls[index],
        )?;
        let old_capture = engine.calls[index].capture;
        engine.calls[index].capture = u32::MAX;
        assert!(engine.lookup_call(index).is_err());
        engine.calls[index].capture = old_capture;
        let old_scope = engine.calls[index].visit.path_scope;
        let other = engine
            .paths
            .state
            .facts
            .scopes
            .iter()
            .enumerate()
            .find(|(position, _)| *position != old_scope.0 as usize)
            .map(|(position, _)| u32::try_from(position).map(crate::resolve::model::ScopeId))
            .transpose()?
            .ok_or("other real scope")?;
        engine.calls[index].visit.path_scope = other;
        assert!(engine.lookup_call(index).is_err());
        engine.calls[index].visit.path_scope = old_scope;
        assert_eq!(engine.lookup_call(index)?, Some(selected));
        assert_eq!(counts(engine), before);
        let source = engine.loader.package.source_core().ok_or("source")?;
        assert!(source.macros.charge(usize::MAX, 0).is_err());
        assert!(engine.lookup_call(index).is_err());
        assert_eq!(counts(engine), before);
        Ok(())
    })
}

#[test]
fn unknown_attribute_blocks_its_actual_owner_before_nested_macro_work() -> TestResult {
    with_engine(
        "#[unknown]fn target(){missing!();}fn answer()->u32{7u32}",
        false,
        |engine| {
            let error = engine.run().err().ok_or("unknown attribute ignored")?;
            assert!(
                error.message.contains("unresolved attribute macro"),
                "{error}"
            );
            assert_eq!(engine.calls.len(), 1);
            assert!(matches!(engine.calls[0].grammar, CallGrammar::Attribute));
            assert_eq!(counts(engine), (0, 0, 1));
            Ok(())
        },
    )
}

#[test]
fn authentic_builtin_attribute_header_cannot_become_an_allowance_frame_by_body_substitution()
-> TestResult {
    let root = r#"#![feature(no_core,lang_items,rustc_attrs,decl_macro,allow_internal_unstable)]#![no_core]
        #[lang="pointee_sized"]trait PointeeSized{}
        #[lang="meta_sized"]trait MetaSized:PointeeSized{}
        #[lang="sized"]trait Sized:MetaSized{}
        #[rustc_builtin_macro(autodiff_forward)]#[allow_internal_unstable(panic_internals)]pub macro actual($item:item){}
        fn probe()->u32{7u32}
        #[allow_internal_unstable(panic_internals)]macro_rules!live{()=>{probe()};}
        fn answer()->u32{live!()}"#;
    let mut report = crate::test_source::source_provider(root, "attribute_frame")?;
    let attribute = report
        .definitions
        .iter()
        .position(|entry| {
            matches!(
                entry.body,
                super::super::Body::Builtin(Builtin::AutodiffForward)
            )
        })
        .ok_or("attribute definition")?;
    let (file, span) = report
        .package
        .units
        .iter()
        .enumerate()
        .find_map(|(file, unit)| {
            unit.configured
                .ast
                .nodes
                .iter()
                .enumerate()
                .find_map(|(index, node)| {
                    if let crate::syntax::ast::Kind::Name(span, name) = node.kind {
                        let actual = name.map_or_else(
                            || {
                                unit.source
                                    .text()
                                    .get(span.start as usize..span.end as usize)
                            },
                            |name| Some(report.package.names.get(name)),
                        );
                        (unit.configured.active[index] && actual == Some("probe"))
                            .then_some((file, span))
                    } else {
                        None
                    }
                })
        })
        .ok_or("actual reached source token")?;
    let site = super::super::Location {
        file: FileId(u32::try_from(file)?),
        span,
    };
    let feature = crate::resolve::bindings::KnownFeature::PanicInternals.name();
    assert!(report.macro_allowances().allows(site, &feature)?);
    report.invocations[0].definition = attribute;
    for forged_body in [false, true] {
        if forged_body {
            report.definitions[attribute].body = super::super::Body::Builtin(Builtin::Line);
        }
        let error = report
            .macro_allowances()
            .allows(site, &feature)
            .err()
            .ok_or("attribute supplied frame")?;
        assert!(
            error
                .message
                .contains("builtin attribute declaration has no executed expansion frame"),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn actual_ordinary_engine_keeps_no_attribute_calls_and_reports_real_cold_storage_layouts()
-> TestResult {
    for source in [
        "fn answer()->u32{7u32}",
        r#"#[doc="text"]#[allow(dead_code)]fn answer()->u32{7u32}"#,
    ] {
        with_engine(source, false, |engine| {
            engine.run()?;
            assert!(engine.calls.is_empty());
            assert_eq!(engine.calls.capacity(), 0);
            assert_eq!(counts(engine), (0, 0, 1));
            assert!(engine.loader.package.source_core().is_none());
            assert!(engine.removed_functions.is_none());
            assert!(
                !engine
                    .paths
                    .state
                    .facts
                    .macro_prelude
                    .values()
                    .any(|definition| {
                        let actual = &engine.paths.state.facts.definitions[definition.0 as usize];
                        engine.loader.package.names.get(actual.name) == Builtin::Test.name()
                    })
            );
            eprintln!(
                "attribute cold layouts report={} macro_target={} lookup_mode={} removal_owner={} removal={}",
                size_of::<super::super::Report>(),
                size_of::<super::super::MacroTarget>(),
                size_of::<crate::resolve::lookup::LookupMode<'_>>(),
                size_of::<super::super::removed_functions::Removals>(),
                size_of::<super::super::removed_functions::Removal>()
            );

            assert_eq!(engine.builtin_definitions.len(), Builtin::ALL.len());
            let slots = engine.builtin_definitions.capacity();
            let payload = slots * size_of::<Option<usize>>();
            eprintln!(
                "attribute layouts kind={} grammar={} bang-context={} call={} engine={} body={} definition={} slot={} actual-slots={slots} actual-slot-payload={payload}",
                size_of::<BuiltinKind>(),
                size_of::<CallGrammar>(),
                size_of::<MacroContext>(),
                size_of::<Call>(),
                size_of::<Engine<'_, NoFiles>>(),
                size_of::<super::super::Body>(),
                size_of::<super::super::Definition>(),
                size_of::<Option<usize>>()
            );
            assert_eq!(size_of::<Builtin>(), 1);
            assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
            assert_eq!(size_of::<crate::syntax::ast::Kind>(), 28);
            assert_eq!(size_of::<super::super::Body>(), 96);
            assert_eq!(size_of::<super::super::Definition>(), 136);
            Ok(())
        })?;
    }
    Ok(())
}
