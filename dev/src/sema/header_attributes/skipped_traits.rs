//! Original trait dispatch policy precedes edition-aware receiver filtering.
use super::{Declarations, DefId, Error, Header, NodeKey, Report, ledger};
use crate::{
    resolve::{
        expansion::MacroAllowanceSession,
        model::{DefKind, Namespace},
    },
    sema::header_attributes::{ContractKnowledge, SourceContractSite},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::{Item, ItemKind},
        token_tree::{Delimiter, Forest, TreeKind},
    },
};
#[cfg(test)]
#[path = "../../../tests/invariants/skipped_dispatch.rs"]
mod tests;

/// These are declaration facts, not trait evidence or permission to skip a receiver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DispatchPolicy {
    pub array: bool,
    pub boxed_slice: bool,
}
pub struct DispatchPolicyView {
    pub source: SourceContractSite,
    pub policy: DispatchPolicy,
}
pub(super) struct Marker {
    owner_slot: usize,
    effect: usize,
    attribute: Attribute,
    item: Item,
    policy: DispatchPolicy,
    definition: Option<DefId>,
}
fn header(facts: &Declarations, key: NodeKey) -> Option<&Header> {
    facts
        .must_use_contracts
        .as_deref()?
        .headers
        .as_deref()?
        .local
        .declarations
        .get(&key)
}
pub(in crate::sema) fn consume(
    report: &Report,
    facts: &mut Declarations,
    (owner, key): (NodeKey, NodeKey),
    (attribute, effect): (Attribute, usize),
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.require_lint_context(report)?;
    facts.reserve_must_use_source(key, attribute.span)?;
    super::reserve_source_contracts(facts, 1, 0)?;
    let error = |message| Error::at(key.file, attribute.span.start, message);
    crate::resolve::bindings::root_options::require_internal(
        report, allowances, key.file, attribute, forest,
    )?;
    let unit = report.package.unit(key.file);
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error("method dispatch policy requires a trait"));
    };
    let item = unit.configured.ast.items[index as usize];
    if !matches!(item.kind, ItemKind::Trait { .. }) {
        return Err(error(
            "method dispatch policy requires its actual trait owner",
        ));
    }
    let policy = parse(report, facts, key, attribute, forest)?;
    if header(facts, key).is_none() {
        super::observe_selected_owner(report, facts, owner, key, true)?;
    }
    let actual =
        header(facts, key).ok_or_else(|| error("method dispatch policy lost its source header"))?;
    super::charge_source_contract_query(facts, actual.owners.len())?;
    let owner_slot = actual
        .owners
        .iter()
        .position(|row| row.key == owner && row.effects.contains(&effect))
        .ok_or_else(|| error("method dispatch policy belongs to another source owner"))?;
    if actual
        .promises
        .as_deref()
        .is_some_and(|row| row.dispatch.is_some())
    {
        return Err(error(
            "duplicate rustc_skip_during_method_dispatch attribute",
        ));
    }
    let bytes = size_of::<Marker>()
        + if actual.promises.is_none() {
            size_of::<super::Promises>()
        } else {
            0
        };
    let actual = ledger(facts, bytes)?
        .local
        .declarations
        .get_mut(&key)
        .ok_or_else(|| error("method dispatch source header disappeared"))?;
    actual.promises.get_or_insert_default().dispatch = Some(Box::new(Marker {
        owner_slot,
        effect,
        attribute,
        item,
        policy,
        definition: None,
    }));
    Ok(())
}
fn parse(
    report: &Report,
    facts: &Declarations,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
) -> Result<DispatchPolicy, Error> {
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let Input::Delimited(group) = attribute.input else {
        return Err(error("method dispatch policy requires a nonempty list"));
    };
    if attribute.unsafe_wrapper
        || !forest.tree(group).is_some_and(|tree| {
            matches!(
                tree.kind,
                TreeKind::Group {
                    delimiter: Delimiter::Parenthesis,
                    ..
                }
            )
        })
    {
        return Err(error(
            "method dispatch policy requires an ordinary parenthesized list",
        ));
    }
    let range = forest
        .children(group)
        .ok_or_else(|| error("missing dispatch policy list"))?;
    if range.start == range.end {
        return Err(error(
            "method dispatch policy requires at least one argument",
        ));
    }
    let mut policy = DispatchPolicy {
        array: false,
        boxed_slice: false,
    };
    let mut index = range.start;
    while index < range.end {
        super::charge_source_contract_query(facts, 1)?;
        let field = match forest.name(report.package.unit(key.file).source.text(), index) {
            Some("array") => &mut policy.array,
            Some("boxed_slice") => &mut policy.boxed_slice,
            _ => {
                return Err(error(
                    "method dispatch policy expects array or boxed_slice words",
                ));
            }
        };
        if std::mem::replace(field, true) {
            return Err(error("duplicate method dispatch policy argument"));
        }
        index += 1;
        if index < range.end {
            if !forest.punctuation(index, crate::lex::token::Punctuation::Comma) {
                return Err(error(
                    "method dispatch policy arguments must be comma-separated words",
                ));
            }
            index += 1;
        }
    }
    Ok(policy)
}
fn current(
    report: &Report,
    facts: &Declarations,
    key: NodeKey,
) -> Result<Option<DispatchPolicyView>, Error> {
    let actual = header(facts, key)
        .ok_or_else(|| Error::unlocated("method dispatch source header is absent"))?;
    super::validate_source(facts, report, key, actual)?;
    let unit = report.package.unit(key.file);
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(Error::unlocated("method dispatch declaration changed"));
    };
    if !matches!(
        unit.configured.ast.items[index as usize].kind,
        ItemKind::Trait { .. }
    ) {
        return Err(Error::unlocated(
            "method dispatch source owner is no longer a trait",
        ));
    }
    let Some(marker) = actual
        .promises
        .as_deref()
        .and_then(|row| row.dispatch.as_deref())
    else {
        return Ok(None);
    };
    let owner = actual
        .owners
        .get(marker.owner_slot)
        .filter(|row| row.effects.contains(&marker.effect))
        .ok_or_else(|| Error::unlocated("method dispatch configured owner changed"))?;
    let retained = unit
        .retained_attributes(marker.effect..marker.effect + 1)
        .next()
        .ok_or_else(|| Error::unlocated("method dispatch attribute disappeared"))?;
    if retained.0 != marker.attribute
        || retained.0.name(retained.1, unit.source.text())
            != Some("rustc_skip_during_method_dispatch")
        || unit.configured.ast.items[index as usize] != marker.item
    {
        return Err(Error::unlocated(
            "method dispatch source attribute or trait changed",
        ));
    }
    Ok(Some(DispatchPolicyView {
        source: SourceContractSite {
            owner: owner.key,
            declaration: key,
            span: marker.attribute.span,
        },
        policy: marker.policy,
    }))
}
pub(super) fn bind(
    report: &Report,
    facts: &mut Declarations,
    id: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    if header(facts, key)
        .and_then(|row| row.promises.as_deref())
        .is_none_or(|row| row.dispatch.is_none())
    {
        return Ok(());
    }
    let actual = &report.resolution.definitions[id.0 as usize];
    if actual.kind == DefKind::SelfType {
        return Ok(());
    }
    super::charge_source_contract_query(facts, 3)?;
    current(report, facts, key)?;
    if actual.kind != DefKind::Trait
        || actual.namespace != Namespace::Type
        || actual.node != Some(key)
    {
        return Err(Error::unlocated(
            "method dispatch policy changed its actual trait identity",
        ));
    }
    let local = &mut ledger(facts, 0)?.local;
    let marker = local
        .declarations
        .get_mut(&key)
        .and_then(|row| row.promises.as_deref_mut())
        .and_then(|row| row.dispatch.as_deref_mut())
        .ok_or_else(|| Error::unlocated("method dispatch marker disappeared"))?;
    if marker.definition.replace(id).is_some() {
        return Err(Error::unlocated("method dispatch trait was bound twice"));
    }
    local.first_skipped_trait.get_or_insert(id);
    Ok(())
}
impl Declarations {
    /// Query original declaration facts; imported absence needs an explicit profile.
    /// # Errors
    /// Rejects changed original owners, syntax, images and exhausted work budgets.
    pub fn source_method_dispatch_policy(
        &self,
        report: &Report,
        id: DefId,
    ) -> Result<ContractKnowledge<DispatchPolicyView>, Error> {
        self.require_lint_context(report)?;
        let actual = report
            .resolution
            .definitions
            .get(id.0 as usize)
            .filter(|row| row.kind == DefKind::Trait)
            .ok_or_else(|| Error::unlocated("method dispatch policy query requires a trait"))?;
        let Some(key) = actual.node else {
            return Ok(ContractKnowledge::Unknown);
        };
        let Some(local) = self
            .must_use_contracts
            .as_deref()
            .and_then(|row| row.headers.as_deref())
            .map(|row| &row.local)
        else {
            return Ok(ContractKnowledge::Unknown);
        };
        super::charge_source_contract_query(self, 4)?;
        if !local.comptime_complete || local.definitions.get(&id) != Some(&key) {
            return Ok(ContractKnowledge::Unknown);
        }
        let value = current(report, self, key)?;
        if header(self, key)
            .and_then(|row| row.promises.as_deref())
            .and_then(|row| row.dispatch.as_deref())
            .is_some_and(|marker| marker.definition != Some(id))
        {
            return Err(Error::unlocated("method dispatch trait binding changed"));
        }
        Ok(value.map_or(
            ContractKnowledge::KnownAbsent,
            ContractKnowledge::KnownPresent,
        ))
    }
    pub(crate) fn require_dispatch_policy_transport(
        &self,
        report: &Report,
    ) -> Result<(), &'static str> {
        if let Some(id) = self
            .must_use_contracts
            .as_deref()
            .and_then(|row| row.headers.as_deref())
            .and_then(|row| row.local.first_skipped_trait)
        {
            if !matches!(
                self.source_method_dispatch_policy(report, id).map_err(
                    |_| "method dispatch policy lost its authenticated trait before export"
                )?,
                ContractKnowledge::KnownPresent(_)
            ) {
                return Err("method dispatch policy lost its completed source contract");
            }
            return Err("method dispatch policy requires authenticated trait metadata transport");
        }
        Ok(())
    }
}
