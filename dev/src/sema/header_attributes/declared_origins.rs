//! The existing owner/declaration pass records positive source completion.
use super::{ContractKnowledge, ContractOrigin, HeaderAttributeView, Ledger, SourceImage};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::{bindings::LangItem, hash::HashMap, retained},
    syntax::{ast::Kind, item::ItemKind},
};
use std::cell::Cell;

struct Header {
    source: SourceImage,
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
    bytes: Cell<usize>,
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
    ledger
        .local
        .bytes
        .set(ledger.local.bytes.get().saturating_add(bytes));
    if ledger.local.bytes.get() > crate::support::crates::MAX_BYTES
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
    if report.package.source_core().is_none() {
        return Ok(());
    }
    if !admitted(report, declaration) {
        reserve_source_contracts(facts, 3, 0)?;
        if !retained_role_site(report, declaration) {
            return Ok(());
        }
    }
    let bytes =
        retained::new_map::<NodeKey, Header>(1) + size_of::<Header>() + 4 * size_of::<Owner>();
    let headers = &mut ledger(facts, bytes)?.local;
    let unit = report.package.unit(declaration.file);
    let header = headers
        .declarations
        .entry(declaration)
        .or_insert_with(|| Header {
            source: SourceImage::capture(unit, declaration.node),
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
            source: SourceImage::capture(unit, declaration.node),
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
    if !admitted(report, key) {
        return Err(Error::at(
            key.file,
            0,
            "source header completion incarnation changed",
        ));
    }
    validate_source(facts, report, key, header)?;
    match report.resolution.definitions[definition.0 as usize].kind {
        DefKind::Function => {
            if facts
                .declaration_header(definition)
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

/// Local role consumers share the existing source-header producer budget.
pub(in crate::sema) fn reserve_source_contracts(
    facts: &mut Declarations,
    steps: usize,
    bytes: usize,
) -> Result<(), Error> {
    let headers = &ledger(facts, bytes)?.local;
    charge_query(headers, steps)
}
pub(in crate::sema) fn charge_source_contract_query(
    facts: &Declarations,
    steps: usize,
) -> Result<(), Error> {
    let headers = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .map(|ledger| &ledger.local)
        .ok_or_else(|| Error::unlocated("source contract query lacks its producer budget"))?;
    charge_query(headers, steps)
}

/// Counted semantic producers retain against the same already authenticated
/// local owner. This adapter cannot create a ledger or borrow imported budgets.
pub(in crate::sema) fn charge_source_retention(
    facts: &Declarations,
    steps: usize,
    bytes: usize,
) -> Result<(), Error> {
    source_payments(facts.must_use_contracts.as_deref())?
        .charge(steps, bytes)
        .map_err(Error::unlocated)
}

/// A disjoint accounting borrow, not permission to read or publish a header.
pub(in crate::sema) struct Payments<'a>(&'a Headers);
pub(in crate::sema) fn source_payments(
    contracts: Option<&crate::sema::must_use::Contracts>,
) -> Result<Payments<'_>, Error> {
    let headers = contracts
        .and_then(|owner| owner.headers.as_deref())
        .filter(|ledger| ledger.local.complete)
        .map(|ledger| &ledger.local)
        .ok_or_else(|| Error::unlocated("source retention lacks its completed local producer"))?;
    let payments = Payments(headers);
    payments.charge(1, 0).map_err(Error::unlocated)?;
    Ok(payments)
}
impl Payments<'_> {
    pub(in crate::sema) fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        self.0.steps.set(self.0.steps.get().saturating_add(steps));
        self.0.bytes.set(self.0.bytes.get().saturating_add(bytes));
        if self.0.steps.get() > crate::support::crates::MAX_COMPONENTS
            || self.0.bytes.get() > crate::support::crates::MAX_BYTES
        {
            return Err("source contract work or retention exceeds profile limit");
        }
        Ok(())
    }
}
fn charge_query(headers: &Headers, steps: usize) -> Result<(), Error> {
    headers.steps.set(headers.steps.get().saturating_add(steps));
    if headers.steps.get() > crate::support::crates::MAX_COMPONENTS {
        return Err(Error::unlocated(
            "source header contract query work exceeds profile limit",
        ));
    }
    Ok(())
}

/// Reuse the genuine completed header's image witness without widening admission.
pub(in crate::sema) fn completed_source_image(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
) -> Result<SourceImage, Error> {
    let view = query(facts, report, definition)?;
    if matches!(view.must_use, ContractKnowledge::Unknown)
        || matches!(view.inline, ContractKnowledge::Unknown)
    {
        return Err(Error::unlocated(
            "source role requires its positive completed header",
        ));
    }
    let site = view
        .origin
        .source()
        .ok_or_else(|| Error::unlocated("source role has no local header origin"))?;
    facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.local.declarations.get(&site.declaration))
        .map(|header| header.source.clone())
        .ok_or_else(|| Error::unlocated("source role lost its completed header witness"))
}

fn validate_source(
    facts: &Declarations,
    report: &Report,
    key: NodeKey,
    header: &Header,
) -> Result<(), Error> {
    let unit = report.package.unit(key.file);
    if !header.source.current(unit, key.node)
        || unit.configured.active.get(key.node.get() as usize) != Some(&true)
    {
        return Err(Error::at(
            key.file,
            0,
            "source header completion incarnation changed",
        ));
    }
    charge_source_contract_query(facts, header.owners.len())?;
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
    Ok(())
}
fn retained_role_site(report: &Report, key: NodeKey) -> bool {
    report
        .resolution
        .source_bindings
        .as_deref()
        .is_some_and(|reserved| {
            [
                LangItem::PointeeTrait,
                LangItem::MetadataType,
                LangItem::DynMetadata,
                LangItem::ControlFlowContinue,
                LangItem::ControlFlowBreak,
            ]
            .into_iter()
            .any(|role| {
                reserved
                    .language(role)
                    .is_some_and(|declaration| declaration.origin == key)
            })
        })
}
/// Associated/nominal role witnesses do not enter attribute completion admission.
pub(in crate::sema) fn pointee_source_image(
    facts: &Declarations,
    report: &Report,
    declaration: crate::resolve::bindings::Declaration,
) -> Result<SourceImage, Error> {
    charge_source_contract_query(facts, 4)?;
    let actual = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or_else(|| Error::unlocated("source Pointee role lost its actual definition"))?;
    let authenticated = report
        .resolution
        .source_bindings
        .as_deref()
        .is_some_and(|reserved| {
            [
                (LangItem::PointeeTrait, DefKind::Trait),
                (LangItem::MetadataType, DefKind::TypeAlias),
                (LangItem::DynMetadata, DefKind::Struct),
            ]
            .into_iter()
            .any(|(role, kind)| reserved.language(role) == Some(declaration) && actual.kind == kind)
        });
    if actual.node != Some(declaration.origin) || !authenticated {
        return Err(Error::unlocated(
            "source Pointee witness has a different actual definition, role or kind",
        ));
    }
    let header = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .filter(|ledger| ledger.local.complete)
        .and_then(|ledger| ledger.local.declarations.get(&declaration.origin))
        .ok_or_else(|| Error::unlocated("source Pointee role lacks its retained owner witness"))?;
    validate_source(facts, report, declaration.origin, header)?;
    Ok(header.source.clone())
}

/// Variant-role evidence shares the existing configured-owner/image producer.
/// It never completes a function/trait attribute contract for this variant.
pub(in crate::sema) fn require_control_flow_variant(
    facts: &Declarations,
    report: &Report,
    declaration: crate::resolve::bindings::Declaration,
) -> Result<(), Error> {
    charge_source_contract_query(facts, 3)?;
    let actual = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or_else(|| Error::unlocated("source control-flow variant lost its definition"))?;
    let reserved = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("source control-flow variant lost its provider"))?;
    if actual.kind != DefKind::Variant
        || actual.node != Some(declaration.origin)
        || ![LangItem::ControlFlowContinue, LangItem::ControlFlowBreak]
            .into_iter()
            .any(|role| reserved.language(role) == Some(declaration))
    {
        return Err(Error::unlocated(
            "source control-flow variant changed its actual role or kind",
        ));
    }
    let header = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .filter(|ledger| ledger.local.complete)
        .and_then(|ledger| ledger.local.declarations.get(&declaration.origin))
        .ok_or_else(|| {
            Error::unlocated("source control-flow variant lacks its retained owner witness")
        })?;
    validate_source(facts, report, declaration.origin, header)
}

/// Replay an already Report-authenticated source seal while the declaration
/// owner holds that Report immutably. This cannot inspect a different Report.
pub(in crate::sema) fn require_recorded_pointee_image(
    facts: &Declarations,
    declaration: crate::resolve::bindings::Declaration,
    expected: &SourceImage,
) -> Result<(), Error> {
    charge_source_contract_query(facts, 1)?;
    let actual = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .filter(|ledger| ledger.local.complete)
        .and_then(|ledger| ledger.local.declarations.get(&declaration.origin))
        .ok_or_else(|| Error::unlocated("source Pointee lost its recorded image witness"))?;
    if !expected.same(&actual.source) {
        return Err(Error::unlocated(
            "source Pointee recorded image belongs to another incarnation",
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(in crate::sema) fn source_contract_storage(
    facts: &Declarations,
) -> Option<(usize, usize, usize, usize, usize, usize)> {
    let headers = &facts
        .must_use_contracts
        .as_deref()?
        .headers
        .as_deref()?
        .local;
    Some((
        headers.steps.get(),
        headers.bytes.get(),
        headers.declarations.len(),
        headers.declarations.capacity(),
        headers
            .declarations
            .values()
            .map(|header| header.owners.capacity())
            .sum(),
        size_of::<Header>(),
    ))
}

/// Read the actual checked local owners; absence never authenticates an import.
pub(in crate::sema) fn marker_trait(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
) -> Result<bool, Error> {
    let key = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .filter(|definition| definition.kind == DefKind::Trait)
        .and_then(|definition| definition.node)
        .ok_or_else(|| Error::unlocated("marker query requires an actual source trait"))?;
    let header = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .filter(|ledger| ledger.local.complete)
        .and_then(|ledger| ledger.local.declarations.get(&key))
        .ok_or_else(|| Error::unlocated("marker query lacks its completed source header"))?;
    validate_header(report, facts, definition, key, header)?;
    let unit = report.package.unit(key.file);
    for owner in &header.owners {
        charge_source_contract_query(facts, owner.effects.len())?;
        for (attribute, forest) in unit.retained_attributes(owner.effects.clone()) {
            if attribute.name(forest, unit.source.text()) == Some("marker") {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
