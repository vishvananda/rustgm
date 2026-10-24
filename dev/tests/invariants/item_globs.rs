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
