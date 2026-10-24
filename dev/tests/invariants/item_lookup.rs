//! Real expansion owners with supplied opaque contexts isolate resolver consumers.
//! Scoped macro source admission remains disabled in this checkpoint.
use super::*;
use crate::{
    expand::{
        config::Config,
        marks::{SyntaxContextId, Transparency},
    },
    resolve::{
        expansion::{self, Report},
        index::State,
        load::{InputFile, Sources},
        lookup::{Lookup, LookupMode},
        model::Origin,
        package::ModuleId,
        paths::Identified,
    },
    support::source::SourceFile,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn input() -> TestResult<(Report, ItemContext)> {
    let source = "mod definition { fn secret() {} macro_rules! make { () => { pub mod generated { pub fn member() {} } } } make!(); } mod caller { fn secret() {} }";
    let mut report = program(source)?;
    let expansion = report
        .invocations
        .first()
        .ok_or("missing real invocation")?
        .context;
    let raw = report.package.marks.apply(
        SyntaxContextId::ROOT,
        expansion,
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    let context = report.package.marks.item(raw)?;
    Ok((report, context))
}
fn program(source: &str) -> TestResult<Report> {
    Ok(expansion::analyze(
        InputFile {
            identity: "item-lookup.rs".into(),
            path: "item-lookup.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)
}
fn module(report: &Report, name: &str) -> TestResult<ModuleId> {
    let index = report
        .package
        .modules
        .iter()
        .position(|module| report.package.names.get(module.name) == name)
        .ok_or("missing module")?;
    Ok(ModuleId(u32::try_from(index)?))
}
fn context(report: &Report, module: ModuleId) -> Context {
    Context {
        module,
        file: report.package.modules[module.0 as usize].file,
        scope: report.resolution.module_scopes[module.0 as usize],
        foreign: None,
    }
}

#[test]
fn actual_definition_scope_and_callers_select_different_private_items() -> TestResult {
    let (mut report, opaque) = input()?;
    let definition = module(&report, "definition")?;
    let caller = module(&report, "caller")?;
    let call = context(&report, caller);
    let name = report.package.names.intern("secret")?;
    let mut state = State::default();
    state.facts = std::mem::take(&mut report.resolution);
    let resolver = state.bind(&mut report.package);
    let introduced = resolver
        .item_lexical(call, name, Namespace::Value, opaque, None)?
        .ok_or("definition hidden")?;
    let captured = resolver
        .item_lexical(call, name, Namespace::Value, ItemContext::ROOT, None)?
        .ok_or("caller hidden")?;
    assert_ne!(introduced.binding.target, captured.binding.target);
    assert_eq!(introduced.privacy, definition);
    assert_eq!(captured.privacy, caller);
    assert!(resolver.visible(introduced.binding.access, introduced.privacy));
    assert!(!resolver.visible(introduced.binding.access, caller));
    assert_eq!(introduced.key.3, ItemContext::ROOT);
    Ok(())
}

#[test]
fn pending_lexical_names_observe_exact_contexts_in_both_reached_scopes() -> TestResult {
    let (mut report, opaque) = input()?;
    let definition = module(&report, "definition")?;
    let caller = module(&report, "caller")?;
    let call = context(&report, caller);
    let definition_scope = report.resolution.module_scopes[definition.0 as usize];
    let name = report.package.names.intern("later")?;
    let mut state = State::default();
    state.facts = std::mem::take(&mut report.resolution);
    let resolver = state.bind(&mut report.package);
    let mut observed = Observations::default();
    assert!(
        resolver
            .item_lexical(call, name, Namespace::Type, opaque, Some(&mut observed))?
            .is_none()
    );
    assert_eq!(
        observed.dependencies,
        vec![
            (call.scope, name, Namespace::Type, opaque),
            (definition_scope, name, Namespace::Type, ItemContext::ROOT),
        ]
    );
    assert!(!observed.dependencies.contains(&(
        call.scope,
        name,
        Namespace::Type,
        ItemContext::ROOT
    )));
    Ok(())
}

#[test]
fn each_qualified_segment_keeps_its_own_context_and_privacy() -> TestResult {
    let (mut report, opaque) = input()?;
    let definition = module(&report, "definition")?;
    let generated = module(&report, "generated")?;
    let call = context(&report, definition);
    let names = [
        report.package.names.intern("generated")?,
        report.package.names.intern("member")?,
    ];
    let scope = report.resolution.module_scopes[generated.0 as usize];
    let original = *report
        .resolution
        .bindings
        .get(scope, names[1], Namespace::Value)
        .ok_or("missing member")?;
    let opponent = report
        .resolution
        .bindings
        .get(
            call.scope,
            report.package.names.intern("secret")?,
            Namespace::Value,
        )
        .copied()
        .ok_or("missing opponent")?;
    report
        .resolution
        .bindings
        .insert_key((scope, names[1], Namespace::Value, opaque), opponent)?;
    let node = report.package.modules[definition.0 as usize].root;
    let mut state = State::default();
    state.facts = std::mem::take(&mut report.resolution);
    let resolver = state.bind(&mut report.package);
    for (suffix, expected) in [
        (ItemContext::ROOT, original.target),
        (opaque, opponent.target),
    ] {
        let contexts = [ItemContext::ROOT, suffix];
        let found = resolver.lookup_path(
            call,
            node,
            false,
            Identified::new(&names, &contexts),
            LookupMode::Import(Namespace::Value),
            None,
        )?;
        let Lookup::Found(binding) = found else {
            return Err("qualified member missing".into());
        };
        assert_eq!(binding.target, expected);
    }
    Ok(())
}

#[test]
fn self_module_uses_the_keyword_context_and_unknown_owners_fail() -> TestResult {
    let (mut report, opaque) = input()?;
    let definition = module(&report, "definition")?;
    let caller = module(&report, "caller")?;
    let mut state = State::default();
    state.facts = std::mem::take(&mut report.resolution);
    let resolver = state.bind(&mut report.package);
    let mut root = ItemContext::ROOT;
    let mut introduced = opaque;
    assert_eq!(resolver.self_module(caller, &mut root)?, caller);
    assert_eq!(resolver.self_module(caller, &mut introduced)?, definition);
    assert_eq!(introduced, ItemContext::ROOT);
    assert!(resolver.self_module(ModuleId(u32::MAX), &mut root).is_err());
    assert!(
        resolver
            .scope_adjustment(ScopeId(u32::MAX), opaque, caller)
            .is_err()
    );
    Ok(())
}

#[test]
fn a_new_exact_binding_satisfies_a_previous_missing_query_without_absence_caching() -> TestResult {
    let (mut report, opaque) = input()?;
    let caller = module(&report, "caller")?;
    let call = context(&report, caller);
    let name = report.package.names.intern("later")?;
    let mut state = State::default();
    state.facts = std::mem::take(&mut report.resolution);
    let resolver = state.bind(&mut report.package);
    assert!(
        resolver
            .item_lexical(call, name, Namespace::Type, opaque, None)?
            .is_none()
    );
    let key = (call.scope, name, Namespace::Type, opaque);
    let target = resolver.facts.crate_root.ok_or("missing root")?;
    resolver.facts.bindings.insert_key(
        key,
        Binding {
            target,
            access: super::super::model::Access::Public,
            origin: Origin::Direct,
            ambiguous: false,
        },
    )?;
    let selected = resolver
        .item_lexical(call, name, Namespace::Type, opaque, None)?
        .ok_or("new binding ignored")?;
    assert_eq!(selected.key, key);
    assert_eq!(selected.binding.target, target);
    Ok(())
}

#[test]
fn definition_scope_rewinds_before_crossing_its_generic_parameter_barrier() -> TestResult {
    let report = program("fn outer<T>() { macro_rules! mark { () => {} } mark!(); }")?;
    let mut report = crate::resolve::analyze_expanded(report)?;
    let invocation = report.invocations.first().ok_or("missing invocation")?;
    let expansion = invocation.context;
    let definition = &report.definitions[invocation.definition];
    let call = Context {
        module: definition.module,
        file: definition.node.ok_or("missing macro source")?.file,
        scope: report
            .resolution
            .introductions
            .definition_scope(expansion)?,
        foreign: None,
    };
    let raw = report.package.marks.apply(
        SyntaxContextId::ROOT,
        expansion,
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    let context = report.package.marks.item(raw)?;
    let name = report.package.names.intern("T")?;
    let mut state = State::default();
    state.facts = std::mem::take(&mut report.resolution);
    let resolver = state.bind(&mut report.package);
    let found = resolver
        .item_lexical(call, name, Namespace::Type, context, None)?
        .ok_or("definition parameter hidden")?;
    assert_eq!(
        resolver.facts.definitions[found.binding.target.0 as usize].kind,
        DefKind::Generic
    );
    assert_eq!(found.key.3, ItemContext::ROOT);
    assert_eq!(found.privacy, call.module);
    Ok(())
}
