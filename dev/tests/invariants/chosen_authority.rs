//! Actual canonical item and published header identity authorize this builtin.
use super::*;
use crate::{
    expand::{config::Config, marks::SyntaxContextId, textual::Scope},
    resolve::{
        self,
        index::Context,
        load::{InputFile, Sources},
        model::ScopeId,
        package::ModuleId,
    },
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
    syntax::{
        ast::{Kind, NodeId},
        item::ItemKind,
    },
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn actual() -> Result<(
    super::super::Package,
    super::super::paths::Paths,
    Vec<super::super::Definition>,
)> {
    let identity = CrateIdentity::new("minicore", "choice_owner")?;
    let context = resolve::bindings::Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    let source = "#![feature(no_core,rustc_attrs,decl_macro)]#![no_core]#[rustc_builtin_macro(cfg_select)]pub macro selected($($t:tt)*){}";
    let mut package = resolve::load::load_with_context(
        InputFile {
            identity: "choice.rs".into(),
            path: "choice.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        &context,
    )?;
    let ast = &package.units[0].configured.ast;
    let (node, item) = ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, node)| match node.kind {
            Kind::Item(item)
                if matches!(
                    ast.items[item as usize].kind,
                    ItemKind::DeclarativeMacro { .. }
                ) =>
            {
                Some((index, ast.items[item as usize]))
            }
            _ => None,
        })
        .ok_or("missing actual scoped item")?;
    let node = NodeId::new(u32::try_from(node)?);
    let owner = ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, parent)| {
            matches!(parent.kind,Kind::Attributed{value,..} if value==node).then_some(index)
        })
        .map(u32::try_from)
        .transpose()?
        .map(NodeId::new)
        .ok_or("missing actual attributed owner")?;
    let mut paths = super::super::paths::Paths::new(&mut package, None)?;
    let (declared, name) = paths.scoped_item(
        &mut package,
        Context {
            module: ModuleId(0),
            file: crate::support::source::FileId(0),
            scope: ScopeId(0),
            foreign: None,
        },
        node,
        Some(owner),
        item,
        (0, ExpansionId(0)),
    )?;
    let definition = super::super::Definition {
        node: Some(NodeKey {
            file: crate::support::source::FileId(0),
            node,
        }),
        module: ModuleId(0),
        name,
        body: Body::Builtin(Builtin::CfgSelect),
        context: SyntaxContextId::ROOT,
        introduction: ExpansionId(0),
        preceding_scope: Scope::ROOT,
        local_inner_macros: false,
        edition: declared.edition,
        edition_panic: false,
        header: declared.header,
    };
    Ok((package, paths, vec![definition]))
}
#[test]
fn selected_builtin_requires_exact_canonical_item_header_role_and_provider() -> Result {
    let (package, mut paths, mut definitions) = actual()?;
    let owner = checked_contract(&package, &paths, &definitions, 0)?;
    assert_eq!(definitions[0].node, Some(owner));
    assert!(checked_contract(&package, &paths, &definitions, 1).is_err());
    definitions[0].body = Body::Builtin(Builtin::CorePanic);
    assert!(checked_contract(&package, &paths, &definitions, 0).is_err());
    definitions[0].body = Body::Builtin(Builtin::CfgSelect);
    let header = definitions[0].header.take();
    assert!(checked_contract(&package, &paths, &definitions, 0).is_err());
    definitions[0].header = header;
    let (foreign, _, mut foreign_definitions) = actual()?;
    assert!(checked_contract(&foreign, &paths, &definitions, 0).is_err());
    let foreign_header = foreign_definitions[0].header.take();
    let header = std::mem::replace(&mut definitions[0].header, foreign_header);
    assert!(checked_contract(&package, &paths, &definitions, 0).is_err());
    definitions[0].header = header;
    let node = definitions[0].node.take();
    assert!(checked_contract(&package, &paths, &definitions, 0).is_err());
    definitions[0].node = node;
    let scope = *paths
        .state
        .facts
        .node_scopes
        .get(&(ModuleId(0), owner.node))
        .ok_or("missing actual scope")?;
    paths
        .state
        .facts
        .node_scopes
        .insert(ModuleId(0), owner.file, owner.node, ScopeId(u32::MAX));
    assert!(checked_contract(&package, &paths, &definitions, 0).is_err());
    paths
        .state
        .facts
        .node_scopes
        .insert(ModuleId(0), owner.file, owner.node, scope);
    checked_contract(&package, &paths, &definitions, 0)?;
    let source = package.source_core().ok_or("source owner")?;
    assert!(source.macros.charge(1_000_001, 0).is_err());
    assert!(
        checked_contract(&package, &paths, &definitions, 0).is_err(),
        "warm identity cannot bypass global failed work"
    );
    Ok(())
}
#[test]
fn invocation_identity_must_be_next_and_parent_and_definition_scope_must_exist() -> Result {
    let (_, mut paths, definitions) = actual()?;
    check_invocation(&paths, 0, ExpansionId(1), ExpansionId(0))?;
    assert!(check_invocation(&paths, 1, ExpansionId(1), ExpansionId(0)).is_err());
    assert!(check_invocation(&paths, 0, ExpansionId(1), ExpansionId(7)).is_err());
    paths.invocation(ExpansionId(1), ExpansionId(0), &definitions[0])?;
    assert!(check_invocation(&paths, 0, ExpansionId(1), ExpansionId(0)).is_err());
    check_invocation(&paths, 1, ExpansionId(2), ExpansionId(1))?;
    eprintln!(
        "cfg_select cold Body={} Definition={} Package={} SourceCore={}",
        size_of::<Body>(),
        size_of::<super::super::Definition>(),
        size_of::<super::super::Package>(),
        size_of::<crate::resolve::bindings::SourceCore>()
    );
    Ok(())
}
