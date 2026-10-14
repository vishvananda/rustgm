//! Index genuine built-in Drop impls and their associated items in linear passes.
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Target},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    syntax::{
        ast::{Kind, TypeFlavor},
        item::{Item, ItemKind, Safety, Visibility},
    },
};
use std::collections::HashMap;

pub(super) struct Header {
    pub source: NodeKey,
    pub self_definition: DefId,
    pub target: NodeKey,
    pub method: Option<DefId>,
}
pub(super) fn item(report: &Report, key: NodeKey) -> Result<Item, Error> {
    let ast = &report.package.unit(key.file).configured.ast;
    let Kind::Item(index) = ast.node(key.node).kind else {
        return Err(Declarations::error(report, key, "missing Drop item syntax"));
    };
    Ok(ast.items[index as usize])
}
pub(super) fn collect(report: &Report, facts: &Declarations) -> Result<Vec<Header>, Error> {
    let Some(drop_trait) = report.resolution.drop_trait else {
        return Ok(Vec::new());
    };
    let mut headers = Vec::new();
    let mut scopes = HashMap::new();
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if definition.kind != DefKind::SelfType {
            continue;
        }
        let Some(source) = definition.node else {
            continue;
        };
        let ItemKind::Impl {
            unsafe_,
            negative,
            trait_path: Some(path),
            ty,
            ..
        } = item(report, source)?.kind
        else {
            continue;
        };
        let ast = &report.package.unit(source.file).configured.ast;
        let Kind::Type {
            flavor: TypeFlavor::Path,
            elements,
        } = ast.node(path).kind
        else {
            continue;
        };
        let Some(&path) = ast.list(elements).first() else {
            continue;
        };
        let path = NodeKey {
            node: path,
            ..source
        };
        if facts.node(path).and_then(|f| f.reference) != Some(Target::Definition(drop_trait)) {
            continue;
        }
        if unsafe_ || negative || crate::sema::lifetimes::arguments(report, path, false)?.is_some()
        {
            return Err(Declarations::error(
                report,
                source,
                "Drop requires a positive safe impl without trait arguments",
            ));
        }
        let self_definition = DefId(
            u32::try_from(index)
                .map_err(|_| Declarations::error(report, source, "too many Drop declarations"))?,
        );
        scopes.insert(definition.scope, headers.len());
        headers.push(Header {
            source,
            self_definition,
            target: NodeKey { node: ty, ..source },
            method: None,
        });
    }
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        let Some(&header) = scopes.get(&definition.scope) else {
            continue;
        };
        if matches!(
            definition.kind,
            DefKind::SelfType | DefKind::Lifetime | DefKind::Generic
        ) {
            continue;
        }
        let header = &mut headers[header];
        if definition.kind != DefKind::Function
            || report.package.names.get(definition.name) != "drop"
            || header.method.is_some()
        {
            return Err(Declarations::error(
                report,
                header.source,
                "Drop requires exactly the drop method and no other associated items",
            ));
        }
        let key = definition.node.ok_or_else(|| {
            Declarations::error(report, header.source, "missing Drop method source")
        })?;
        validate_method(report, key)?;
        header.method =
            Some(DefId(u32::try_from(index).map_err(|_| {
                Declarations::error(report, key, "too many Drop methods")
            })?));
    }
    Ok(headers)
}
fn validate_method(report: &Report, key: NodeKey) -> Result<(), Error> {
    let method = item(report, key)?;
    let ItemKind::Function {
        flags, abi, body, ..
    } = method.kind
    else {
        return Err(Declarations::error(report, key, "missing Drop method"));
    };
    if flags.safety != Safety::Default
        || flags.constant
        || flags.asynchronous
        || !crate::sema::declarations::rust_abi(report, key, abi)?
        || body.is_none()
        || method.visibility != Visibility::Private
    {
        return Err(Declarations::error(
            report,
            key,
            "Drop method must be a safe, non-const, non-async Rust method with an inherited visibility and a body",
        ));
    }
    Ok(())
}
