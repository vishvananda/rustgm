use super::{Binding, ItemContext, Namespace, Origin, Work};
use crate::{
    expand::{
        config::Config,
        marks::{ExpansionId, SyntaxContextId, Transparency},
    },
    resolve::{
        index::State,
        load::{InputFile, Sources},
        model::{Access, DefId},
    },
    support::source::SourceFile,
};
use std::path::{Path, PathBuf};

struct Empty;
impl Sources for Empty {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}

#[test]
fn glob_withdrawal_and_direct_masking_keep_sibling_contexts_independent()
-> Result<(), Box<dyn std::error::Error>> {
    let mut package = crate::resolve::load::load(
        InputFile {
            identity: PathBuf::from("source.rs"),
            path: PathBuf::from("source.rs"),
            source: SourceFile::new(b"mod a {} use a::*;".to_vec())?,
        },
        &mut Empty,
        &Config::default(),
    )?;
    let name = package.names.intern("shared")?;
    let raw = package.marks.apply(
        SyntaxContextId::ROOT,
        ExpansionId(1),
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    let context = package.marks.item(raw)?;
    let mut state = State::default();
    let mut resolver = state.bind(&mut package);
    resolver.collect()?;
    let import = resolver
        .facts
        .imports
        .first()
        .ok_or("missing glob")?
        .clone();
    let root = (import.scope, name, Namespace::Value, ItemContext::ROOT);
    let opaque = (root.0, root.1, root.2, context);
    let glob = Binding {
        target: DefId(7),
        access: Access::Public,
        origin: Origin::Glob(0),
        ambiguous: false,
    };
    let direct = Binding {
        target: DefId(9),
        origin: Origin::Direct,
        ..glob
    };
    let mut work = Work::new(resolver.facts);
    work.observe();
    // Identical spelling, namespace and import origin; only contexts differ.
    resolver.contribute(0, root, Some(glob), &mut work)?;
    resolver.contribute(0, opaque, Some(glob), &mut work)?;
    assert_eq!(work.provided[0].len(), 2);
    assert_eq!(work.candidates.len(), 2);
    resolver.publish(root, Some(direct), &mut work)?;
    assert_eq!(resolver.facts.bindings.get_key(opaque), Some(&glob));
    let before = work.events.len();
    resolver.disconnect_glob(0, &import, &mut work)?;
    assert_eq!(resolver.facts.bindings.get_key(root), Some(&direct));
    assert_eq!(resolver.facts.bindings.get_key(opaque), None);
    assert!(work.provided[0].is_empty());
    assert!(work.candidates.values().all(Vec::is_empty));
    assert_eq!(&work.events[before..], &[root, opaque]);
    // Reconnection preserves both key identities and the direct root mask.
    let created = resolver.facts.bindings.created();
    resolver.contribute(0, opaque, Some(glob), &mut work)?;
    resolver.contribute(0, root, Some(glob), &mut work)?;
    assert_eq!(resolver.facts.bindings.created(), created);
    assert_eq!(resolver.facts.bindings.get_key(root), Some(&direct));
    assert_eq!(resolver.facts.bindings.get_key(opaque), Some(&glob));
    Ok(())
}

#[test]
fn actual_glob_journals_remap_names_and_withdraw_only_the_mapped_context()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "pub fn shared() {} pub fn other() {} mod a {} mod b { use crate::a::*; } macro_rules! mark { () => {} } mark!();";
    let mut report = crate::resolve::expansion::analyze(
        InputFile {
            identity: "glob.rs".into(),
            path: "glob.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut Empty,
        &Config::default(),
    )?;
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
    let name = report.package.names.intern("shared")?;
    let other = report.package.names.intern("other")?;
    let binding = *report
        .resolution
        .bindings
        .get(super::ScopeId(0), name, Namespace::Value)
        .ok_or("missing source function")?;
    let replacement = *report
        .resolution
        .bindings
        .get(super::ScopeId(0), other, Namespace::Value)
        .ok_or("missing replacement")?;
    let source_module = report
        .package
        .modules
        .iter()
        .position(|module| report.package.names.get(module.name) == "a")
        .ok_or("missing source module")?;
    let source_scope = report.resolution.module_scopes[source_module];
    let import = report
        .resolution
        .imports
        .first_mut()
        .ok_or("missing glob")?;
    import.contexts = Some(std::sync::Arc::new(crate::resolve::model::ImportContexts {
        segments: vec![ItemContext::ROOT; import.path.len()].into_boxed_slice(),
        glob: context,
    }));
    let root = (import.scope, name, Namespace::Value, ItemContext::ROOT);
    let opaque = (import.scope, name, Namespace::Value, context);
    let source_key = (source_scope, name, Namespace::Value, ItemContext::ROOT);
    report.resolution.bindings.insert_key(root, replacement)?;
    report.resolution.bindings.insert_key(source_key, binding)?;
    let mut state = State::default();
    state.facts = std::mem::take(&mut report.resolution);
    let mut resolver = state.bind(&mut report.package);
    let mut work = Work::new(resolver.facts);
    resolver.drive_imports(&mut work)?;
    assert!(work.errors.iter().all(Option::is_none));
    assert_eq!(
        resolver.facts.bindings.get_key(opaque).map(|b| b.target),
        Some(binding.target)
    );
    assert_eq!(resolver.facts.bindings.get_key(root), Some(&replacement));
    resolver.publish(source_key, Some(replacement), &mut work)?;
    resolver.drive_imports(&mut work)?;
    assert_eq!(
        resolver.facts.bindings.get_key(opaque).map(|b| b.target),
        Some(replacement.target)
    );
    resolver.publish(source_key, None, &mut work)?;
    resolver.drive_imports(&mut work)?;
    assert_eq!(resolver.facts.bindings.get_key(opaque), None);
    assert_eq!(resolver.facts.bindings.get_key(root), Some(&replacement));
    assert!(work.provided[0].is_empty());
    assert!(work.errors.iter().all(Option::is_none));
    Ok(())
}
