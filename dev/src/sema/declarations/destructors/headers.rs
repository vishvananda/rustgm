//! Index genuine selected Drop impls and their associated items in linear passes.
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Target},
        package::Error,
        unit::NodeKey,
    },
    sema::{associated::OwnerKind, model::Declarations},
    syntax::{
        ast::Kind,
        item::{Item, ItemKind, Safety, Visibility},
    },
};

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
    let selected = if let Some(catalog) = facts.source_bindings.as_deref() {
        catalog
            .language(crate::support::bindings::LangItem::Drop)
            .map(|d| d.definition)
    } else {
        report.resolution.drop_trait
    };
    let Some(drop_trait) = selected else {
        return Ok(Vec::new());
    };
    let mut headers = Vec::new();
    let mut owners = vec![None; facts.associated.owners().len()];
    for index in 0..facts.associated.owners().len() {
        let Some(owner) = facts
            .associated
            .source_at(index)
            .map_err(Error::unlocated)?
        else {
            continue;
        };
        let OwnerKind::Impl {
            unsafe_,
            polarity,
            trait_type: Some(path),
            target,
            constant,
        } = owner.kind
        else {
            continue;
        };
        let source = owner.source;
        let path = NodeKey {
            node: path.node,
            ..source
        };
        if facts.node(path).and_then(|f| f.reference) != Some(Target::Definition(drop_trait)) {
            continue;
        }
        if constant
            || unsafe_
            || polarity != crate::sema::associated::ImplPolarity::Positive
            || crate::sema::lifetimes::arguments(report, path, false)?.is_some()
        {
            return Err(Declarations::error(
                report,
                source,
                "Drop requires a positive safe impl without trait arguments",
            ));
        }
        owners[index] = Some(headers.len());
        headers.push(Header {
            source,
            self_definition: owner.self_definition,
            target,
            method: None,
        });
    }
    for member in facts.associated.members() {
        let Some(header) = owners[member.owner.0 as usize] else {
            continue;
        };
        let definition = &report.resolution.definitions[member.definition.0 as usize];
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
        header.method = Some(member.definition);
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
