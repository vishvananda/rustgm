//! The existing owner/declaration pass records positive source completion.
use super::{ContractKnowledge, ContractOrigin, HeaderAttributeView, Ledger};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::{hash::HashMap, retained, source::OriginalImage},
    syntax::{ast::Kind, item::ItemKind},
};
use std::{cell::Cell, sync::Arc};

struct Header {
    image: Arc<OriginalImage>,
    syntax: Kind,
    owners: Vec<Owner>,
}
struct Owner {
    key: NodeKey,
    syntax: Kind,
    effects: std::ops::Range<usize>,
}
#[derive(Default)]
pub(super) struct Headers {
    declarations: HashMap<NodeKey, Header>,
    definitions: HashMap<DefId, NodeKey>,
    complete: bool,
    bytes: usize,
    pub(super) steps: Cell<usize>,
}
fn ledger(facts: &mut Declarations, bytes: usize) -> Result<&mut Ledger, Error> {
    let ledger = facts
        .must_use_contracts
        .get_or_insert_default()
        .headers
        .get_or_insert_default();
    ledger
        .local
        .steps
        .set(ledger.local.steps.get().saturating_add(1));
    ledger.local.bytes = ledger.local.bytes.saturating_add(bytes);
    if ledger.local.bytes > crate::support::crates::MAX_BYTES
        || ledger.local.steps.get() > crate::support::crates::MAX_COMPONENTS
    {
        return Err(Error::unlocated(
            "source header attribute producer exceeds storage budget",
        ));
    }
    Ok(ledger)
}
pub(in crate::sema) fn observe_owner(
    report: &Report,
    facts: &mut Declarations,
    owner: NodeKey,
    declaration: NodeKey,
) -> Result<(), Error> {
    if report.package.source_core().is_none() || !admitted(report, declaration) {
        return Ok(());
    }
    let bytes =
        retained::new_map::<NodeKey, Header>(1) + size_of::<Header>() + 4 * size_of::<Owner>();
    let headers = &mut ledger(facts, bytes)?.local;
    let unit = report.package.unit(declaration.file);
    let header = headers
        .declarations
        .entry(declaration)
        .or_insert_with(|| Header {
            image: Arc::clone(unit.source.original_image()),
            syntax: unit.configured.ast.node(declaration.node).kind,
            owners: Vec::new(),
        });
    let index = unit
        .owner_indices
        .get(owner.node.get() as usize)
        .copied()
        .flatten()
        .ok_or_else(|| Error::at(owner.file, 0, "source header owner was not configured"))?;
    let configured = &unit.configured.owners[index.get() as usize - 1];
    header.owners.push(Owner {
        key: owner,
        syntax: unit.configured.ast.node(owner.node).kind,
        effects: configured.attributes.clone(),
    });
    Ok(())
}
pub(in crate::sema) fn bind_definition(
    report: &Report,
    facts: &mut Declarations,
    definition: DefId,
    declaration: NodeKey,
) -> Result<(), Error> {
    if report.package.source_core().is_none() || !admitted(report, declaration) {
        return Ok(());
    }
    let bytes = retained::new_map::<DefId, NodeKey>(1)
        + retained::new_map::<NodeKey, Header>(1)
        + size_of::<Header>();
    let headers = &mut ledger(facts, bytes)?.local;
    let unit = report.package.unit(declaration.file);
    headers
        .declarations
        .entry(declaration)
        .or_insert_with(|| Header {
            image: Arc::clone(unit.source.original_image()),
            syntax: unit.configured.ast.node(declaration.node).kind,
            owners: Vec::new(),
        });
    if headers
        .definitions
        .insert(definition, declaration)
        .is_some()
    {
        return Err(Error::at(
            declaration.file,
            0,
            "duplicate source header completion identity",
        ));
    }
    Ok(())
}
pub(in crate::sema) fn finish(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    if report.package.source_core().is_some() {
        ledger(facts, 0)?.local.complete = true;
    }
    Ok(())
}
fn admitted(report: &Report, key: NodeKey) -> bool {
    let unit = report.package.unit(key.file);
    unit.include_context.is_none()
        && matches!(unit.configured.ast.node(key.node).kind,
        Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind,
            ItemKind::Function { .. } | ItemKind::Trait { .. }))
}
pub(super) fn query<'a>(
    facts: &'a Declarations,
    report: &Report,
    definition: DefId,
) -> Result<HeaderAttributeView<'a>, Error> {
    let actual = &report.resolution.definitions[definition.0 as usize];
    let key = actual
        .node
        .ok_or_else(|| Error::unlocated("source header lost its declaration"))?;
    let headers = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .map(|ledger| &ledger.local);
    if let Some(headers) = headers {
        headers.steps.set(headers.steps.get().saturating_add(1));
        if headers.steps.get() > crate::support::crates::MAX_COMPONENTS {
            return Err(Error::at(
                key.file,
                0,
                "source header contract query work exceeds profile limit",
            ));
        }
    }
    let unit = report
        .package
        .units
        .get(key.file.0 as usize)
        .ok_or_else(|| Error::at(key.file, 0, "source header completion lost its file"))?;
    let node = unit
        .configured
        .ast
        .nodes
        .get(key.node.get() as usize)
        .ok_or_else(|| Error::at(key.file, 0, "source header completion lost its node"))?;
    let origin = ContractOrigin::source_site(key, key, node.span);
    let Some(header) = headers
        .filter(|headers| headers.complete)
        .and_then(|headers| {
            headers
                .definitions
                .get(&definition)
                .filter(|node| **node == key)
                .and_then(|_| headers.declarations.get(&key))
        })
    else {
        return Ok(HeaderAttributeView {
            origin,
            must_use: ContractKnowledge::Unknown,
            inline: ContractKnowledge::Unknown,
        });
    };
    validate_header(report, facts, definition, key, header)?;
    let must_use = facts.must_use_contract(report, definition)?.map_or(
        ContractKnowledge::KnownAbsent,
        ContractKnowledge::KnownPresent,
    );
    let inline = facts
        .inline_hint_for_definition(report, definition)?
        .map_or(
            ContractKnowledge::KnownAbsent,
            ContractKnowledge::KnownPresent,
        );
    Ok(HeaderAttributeView {
        origin,
        must_use,
        inline,
    })
}

fn validate_header(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    key: NodeKey,
    header: &Header,
) -> Result<(), Error> {
    let unit = report.package.unit(key.file);
    let node = unit.configured.ast.node(key.node);
    if !admitted(report, key)
        || !Arc::ptr_eq(&header.image, unit.source.original_image())
        || header.syntax != node.kind
        || unit.configured.active.get(key.node.get() as usize) != Some(&true)
    {
        return Err(Error::at(
            key.file,
            0,
            "source header completion incarnation changed",
        ));
    }
    for owner in &header.owners {
        let actual = unit
            .owner_indices
            .get(owner.key.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1));
        if actual.is_none_or(|actual| {
            !actual.enabled || actual.node != owner.key.node || actual.attributes != owner.effects
        }) || unit
            .configured
            .ast
            .nodes
            .get(owner.key.node.get() as usize)
            .is_none_or(|node| node.kind != owner.syntax)
        {
            return Err(Error::at(
                owner.key.file,
                0,
                "source header configured owner changed",
            ));
        }
        if !owner.effects.is_empty() {
            report
                .lint_producer()
                .ok_or_else(|| Error::at(owner.key.file, 0, "source header owner has no producer"))?
                .validate_owner(&report.package, owner.key)
                .map_err(Error::from)?;
        }
    }
    match report.resolution.definitions[definition.0 as usize].kind {
        DefKind::Function => {
            if facts
                .function_header(definition)
                .map_err(Error::unlocated)?
                .is_none_or(|header| header.declaration.source() != Some(key))
            {
                return Err(Error::at(
                    key.file,
                    0,
                    "source attribute header changed its semantic function",
                ));
            }
        }
        DefKind::Trait => {
            if facts
                .generics
                .traits
                .as_deref()
                .and_then(|traits| traits.header(crate::types::ItemId(definition.0)))
                .is_none()
            {
                return Err(Error::at(
                    key.file,
                    0,
                    "source attribute header has no completed semantic trait",
                ));
            }
        }
        _ => {
            return Err(Error::at(
                key.file,
                0,
                "source attribute query requires a function or trait",
            ));
        }
    }
    Ok(())
}
