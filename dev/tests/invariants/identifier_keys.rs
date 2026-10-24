//! Parsed declaration keys consume supplied token provenance, not item keywords.
//! These structural controls do not claim end-to-end scoped macro expansion.
use super::{DefKind, ExpansionId, ItemContext, Namespace};
use crate::{
    expand::{
        config::Config,
        marks::{SyntaxContextId, Transparency},
        stream::{Location, Origin},
    },
    resolve::{
        self,
        load::{InputFile, Sources},
        package::Package,
        unit::FileId,
    },
    support::source::SourceFile,
    syntax::{ast::Kind, token_tree::Span},
};
use std::path::Path;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn package(source: &str) -> TestResult<Package> {
    Ok(resolve::load::load(
        InputFile {
            identity: "keys.rs".into(),
            path: "keys.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)
}
fn mark(package: &mut Package, expansion: u32) -> TestResult<SyntaxContextId> {
    Ok(package.marks.apply(
        SyntaxContextId::ROOT,
        ExpansionId(expansion),
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?)
}
fn provenance(package: &mut Package, span: Span, context: SyntaxContextId) {
    package.units[0].origins.push(Origin {
        generated: span,
        original: Location {
            file: FileId(0),
            span,
        },
        context: Some(context),
        introduction: ExpansionId(1),
        joint: None,
    });
    package.units[0]
        .origins
        .sort_by_key(|origin| origin.generated.start);
}

#[test]
fn source_identifiers_separate_root_and_opaque_items_in_the_same_scope() -> TestResult {
    let mut package = package("fn same(){} fn same(){}")?;
    let keyword = mark(&mut package, 1)?;
    let literal = mark(&mut package, 2)?;
    let ast = &package.units[0].configured.ast;
    let name = ast.items[1].name.ok_or("second actual identifier")?;
    let start = ast
        .nodes
        .iter()
        .find(|node| matches!(node.kind, Kind::Item(1)))
        .ok_or("second actual item")?
        .span
        .start;
    provenance(
        &mut package,
        Span {
            start,
            end: start + 2,
        },
        keyword,
    );
    provenance(&mut package, name, literal);
    let facts = resolve::analyze(&mut package)?;
    let scope = facts.module_scopes[0];
    let name = package.name(FileId(0), name)?;
    let root = facts
        .bindings
        .get(scope, name, Namespace::Value)
        .ok_or("captured root binding")?;
    let context = package.marks.item(literal)?;
    let opaque = facts
        .bindings
        .get_key((scope, name, Namespace::Value, context))
        .ok_or("literal opaque binding")?;
    assert_ne!(root.target, opaque.target);
    assert!(facts.bindings.has_contextual_items());
    assert!(
        facts
            .bindings
            .get_key((scope, name, Namespace::Value, package.marks.item(keyword)?,))
            .is_none(),
        "item keyword context cannot rename its identifier"
    );
    Ok(())
}

#[test]
fn two_source_declarations_with_the_same_complete_key_still_conflict() -> TestResult {
    let mut package = package("fn same(){} fn same(){}")?;
    let context = mark(&mut package, 1)?;
    let names = package.units[0]
        .configured
        .ast
        .items
        .iter()
        .filter_map(|item| item.name)
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 2);
    for name in names {
        provenance(&mut package, name, context);
    }
    let error = resolve::analyze(&mut package)
        .err()
        .ok_or("duplicate full key accepted")?;
    assert!(error.message.contains("duplicate value name"), "{error}");
    Ok(())
}

#[test]
fn const_parameters_use_item_context_but_implicit_self_keeps_root_identity() -> TestResult {
    let mut package = package("trait Named<const N:usize>{fn method();}")?;
    let context = mark(&mut package, 1)?;
    let generic = package.units[0].configured.ast.generics[0].name;
    let item = package.units[0]
        .configured
        .ast
        .items
        .iter()
        .find(|item| matches!(item.kind, crate::syntax::item::ItemKind::Trait { .. }))
        .and_then(|item| item.name)
        .ok_or("actual trait identifier")?;
    provenance(&mut package, item, context);
    provenance(&mut package, generic, context);
    let facts = resolve::analyze(&mut package)?;
    let mut seen = 0;
    for definition in &facts.definitions {
        let (namespace, expected) = match definition.kind {
            DefKind::Generic => (Namespace::Value, package.marks.item(context)?),
            DefKind::SelfType => (Namespace::Type, ItemContext::ROOT),
            _ => continue,
        };
        seen += 1;
        assert_eq!(definition.namespace, namespace);
        assert!(
            facts
                .bindings
                .get_key((definition.scope, definition.name, namespace, expected,))
                .is_some()
        );
        let other = if expected == ItemContext::ROOT {
            package.marks.item(context)?
        } else {
            ItemContext::ROOT
        };
        assert!(
            facts
                .bindings
                .get_key((definition.scope, definition.name, namespace, other,))
                .is_none()
        );
    }
    assert_eq!(
        seen, 2,
        "actual const parameter and implicit Self are both observed"
    );
    Ok(())
}

#[test]
fn an_import_alias_keeps_its_own_identifier_context_when_published() -> TestResult {
    let mut package = package("mod a{pub fn original(){}}use a::original as same;fn same(){}")?;
    let raw = mark(&mut package, 1)?;
    let rename = package.units[0]
        .configured
        .ast
        .use_trees
        .iter()
        .find_map(|tree| tree.rename)
        .ok_or("actual import alias")?;
    let span = package.units[0].configured.ast.node(rename).span;
    provenance(&mut package, span, raw);
    let facts = resolve::analyze(&mut package)?;
    let import = facts.imports.first().ok_or("collected import")?;
    let (name, context) = import.alias.ok_or("retained alias")?;
    assert_eq!(context, package.marks.item(raw)?);
    let root = facts
        .bindings
        .get(import.scope, name, Namespace::Value)
        .ok_or("root spelling peer")?;
    let alias = facts
        .bindings
        .get_key((import.scope, name, Namespace::Value, context))
        .ok_or("opaque alias publication")?;
    assert_ne!(root.target, alias.target);
    assert_eq!(
        package
            .names
            .get(facts.definitions[alias.target.0 as usize].name),
        "original"
    );
    Ok(())
}

fn named_span(package: &Package, spelling: &str) -> TestResult<Span> {
    let unit = &package.units[0];
    for node in &unit.configured.ast.nodes {
        if let Kind::Name(span, _) = node.kind
            && unit.spelling(span).map_err(|error| format!("{error:?}"))? == spelling
        {
            return Ok(span);
        }
    }
    Err(format!("missing parsed name {spelling}").into())
}

#[test]
fn grouped_imports_keep_each_segment_alias_and_star_context_independent() -> TestResult {
    let mut package = package("use root::{left::{a as renamed, *}, right, tail::{leaf}};")?;
    let left = mark(&mut package, 1)?;
    let element = mark(&mut package, 2)?;
    let star = mark(&mut package, 3)?;
    let alias = mark(&mut package, 4)?;
    for (name, context) in [("left", left), ("a", element), ("renamed", alias)] {
        let span = named_span(&package, name)?;
        provenance(&mut package, span, context);
    }
    let span = package.units[0]
        .configured
        .ast
        .use_trees
        .iter()
        .find_map(|tree| tree.glob)
        .ok_or("actual star")?;
    provenance(&mut package, span, star);
    let mut state = resolve::index::State::default();
    state.bind(&mut package).collect()?;
    let imports = &state.facts.imports;
    assert_eq!(imports.len(), 4);
    let names = imports
        .iter()
        .map(|import| {
            import
                .path
                .iter()
                .map(|name| package.names.get(*name))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            vec!["root", "left", "a"],
            vec!["root", "left"],
            vec!["root", "right"],
            vec!["root", "tail", "leaf"]
        ]
    );
    let contexts = imports[0].contexts.as_ref().ok_or("mixed leaf contexts")?;
    assert_eq!(
        contexts.segments.as_ref(),
        &[
            ItemContext::ROOT,
            package.marks.item(left)?,
            package.marks.item(element)?
        ]
    );
    assert_eq!(contexts.glob, ItemContext::ROOT);
    assert_eq!(
        imports[0].alias.map(|(_, context)| context),
        Some(package.marks.item(alias)?)
    );
    let glob = imports[1].contexts.as_ref().ok_or("glob contexts")?;
    assert_eq!(
        glob.segments.as_ref(),
        &[ItemContext::ROOT, package.marks.item(left)?]
    );
    assert_eq!(glob.glob, package.marks.item(star)?);
    assert!(imports[1].glob);
    assert!(imports[2].contexts.is_none());
    assert!(
        imports[3].contexts.is_none(),
        "opaque sibling must not leak into later groups"
    );
    let retry = imports[0].clone();
    assert!(std::sync::Arc::ptr_eq(&retry.path, &imports[0].path));
    assert!(std::sync::Arc::ptr_eq(
        retry.contexts.as_ref().ok_or("retry contexts")?,
        contexts
    ));
    Ok(())
}

#[test]
fn an_opaque_star_retains_root_segments_without_relabeling_them() -> TestResult {
    let mut package = package("use outer::inner::*;")?;
    let context = mark(&mut package, 1)?;
    let span = package.units[0]
        .configured
        .ast
        .use_trees
        .iter()
        .find_map(|tree| tree.glob)
        .ok_or("actual star")?;
    provenance(&mut package, span, context);
    let mut state = resolve::index::State::default();
    state.bind(&mut package).collect()?;
    let import = state.facts.imports.first().ok_or("collected glob")?;
    let contexts = import.contexts.as_ref().ok_or("star context carrier")?;
    assert_eq!(
        contexts.segments.as_ref(),
        &[ItemContext::ROOT, ItemContext::ROOT]
    );
    assert_eq!(contexts.glob, package.marks.item(context)?);
    Ok(())
}
