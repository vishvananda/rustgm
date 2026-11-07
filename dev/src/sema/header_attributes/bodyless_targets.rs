//! No-body advisories use bound source ownership, never executable permissions.
use super::{
    Declarations, DefId, DefKind, Error, Kind, NodeKey, Report, charge_source_contract_query,
    validate_source,
};
use crate::{
    resolve::model::Namespace,
    sema::{associated::OwnerKind, must_use::WarningCode},
    syntax::{attribute::Attribute, item::ItemKind},
};

pub(in crate::sema) fn capture(
    report: &Report,
    facts: &mut Declarations,
    owner: NodeKey,
    declaration: NodeKey,
) -> Result<(), Error> {
    let ast = &report.package.unit(declaration.file).configured.ast;
    if !matches!(ast.node(declaration.node).kind, Kind::Item(index)
        if matches!(ast.items[index as usize].kind, ItemKind::Function { body: None, .. }))
    {
        return Err(Error::at(
            declaration.file,
            0,
            "cold prototype capture requires its no-body function",
        ));
    }
    // SourceCore's normal owner pass already retained this exact row. Ordinary
    // local declarations use that same producer only when this advisory needs it.
    if report.package.source_core().is_none() {
        super::observe_selected_owner(report, facts, owner, declaration, true)?;
    }
    Ok(())
}

fn first_cold(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    key: NodeKey,
) -> Result<Option<(NodeKey, Attribute)>, Error> {
    let Some(headers) = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .map(|ledger| &ledger.local)
    else {
        return Ok(None);
    };
    let Some(header) = headers.declarations.get(&key) else {
        return Ok(None);
    };
    if headers.definitions.get(&definition) != Some(&key) {
        return Err(Error::at(
            key.file,
            0,
            "cold prototype lacks its bound declaration",
        ));
    }
    validate_source(facts, report, key, header)?;
    let unit = report.package.unit(key.file);
    for owner in &header.owners {
        charge_source_contract_query(facts, owner.effects.len())?;
        for (attribute, forest) in unit.retained_attributes(owner.effects.clone()) {
            if attribute.name(forest, unit.source.text()) == Some("cold") {
                return Ok(Some((owner.key, attribute)));
            }
        }
    }
    Ok(None)
}

pub(in crate::sema) fn bind(
    report: &Report,
    facts: &mut Declarations,
    definition: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    let Some(actual) = report.resolution.definitions.get(definition.0 as usize) else {
        return Err(Error::at(
            key.file,
            0,
            "cold prototype definition disappeared",
        ));
    };
    let ast = &report.package.unit(key.file).configured.ast;
    if actual.kind != DefKind::Function
        || !matches!(ast.node(key.node).kind, Kind::Item(index)
        if matches!(ast.items[index as usize].kind, ItemKind::Function { body: None, .. }))
    {
        return Ok(());
    }
    let Some((owner, attribute)) = first_cold(report, facts, definition, key)? else {
        return Ok(());
    };
    let module = report
        .resolution
        .scopes
        .get(actual.scope.0 as usize)
        .ok_or_else(|| Error::at(key.file, 0, "cold prototype scope disappeared"))?
        .module;
    if actual.node != Some(key)
        || actual.namespace != Namespace::Value
        || report
            .resolution
            .item_definitions
            .get(&(module, key.node, Namespace::Value))
            != Some(&definition)
    {
        return Err(Error::at(
            key.file,
            0,
            "cold prototype changed its actual function identity",
        ));
    }
    if required_trait_method(report, facts, definition, key)? {
        facts.must_use_diagnostic(
            report,
            owner,
            attribute.span.start,
            0,
            WarningCode::UnusedAttributes,
            "cold attribute has no effect on a required trait method".into(),
        )?;
    }
    Ok(())
}

fn required_trait_method(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    key: NodeKey,
) -> Result<bool, Error> {
    let error = || {
        Error::at(
            key.file,
            0,
            "cold no-body function requires its actual trait or foreign owner",
        )
    };
    if let Some(block) = report.resolution.foreign_owners.get(&definition) {
        let unit = report.package.unit(block.file);
        if !report.resolution.foreign_blocks.contains(block)
            || unit.configured.active.get(block.node.get() as usize) != Some(&true)
            || !matches!(unit.configured.ast.node(block.node).kind, Kind::Item(index)
                if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::ExternBlock { .. }))
        {
            return Err(error());
        }
        return Ok(false);
    }
    let owner = facts
        .associated
        .owner(definition)
        .and_then(|id| facts.associated.get(id))
        .ok_or_else(error)?;
    let OwnerKind::Trait {
        definition: parent, ..
    } = owner.kind
    else {
        return Err(error());
    };
    let source = owner.source.ok_or_else(error)?;
    let parent = report
        .resolution
        .definitions
        .get(parent.0 as usize)
        .ok_or_else(error)?;
    let member = &report.resolution.definitions[definition.0 as usize];
    let unit = report.package.unit(source.file);
    let ast = &unit.configured.ast;
    if parent.kind != DefKind::Trait
        || parent.node != Some(source)
        || parent.members != Some(member.scope)
        || unit.configured.active.get(source.node.get() as usize) != Some(&true)
        || !matches!(ast.node(source.node).kind, Kind::Item(index)
            if matches!(ast.items[index as usize].kind, ItemKind::Trait { .. }))
    {
        return Err(error());
    }
    Ok(true)
}
