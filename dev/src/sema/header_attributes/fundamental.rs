//! Fundamental locality and trait knowability belong to real defining owners.
use super::{Declarations, DefId, Error, Header, NodeKey, Report, ledger};
use crate::{
    resolve::{bindings::FeatureName, expansion::MacroAllowanceSession, model::DefKind},
    sema::header_attributes::{ContractKnowledge, SourceContractSite},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::ItemKind,
        token_tree::Forest,
    },
};

pub(super) struct Marker {
    owner_slot: usize,
    effect: usize,
    attribute: Attribute,
    syntax: ItemKind,
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
pub(super) fn marked(facts: &Declarations, key: NodeKey) -> bool {
    header(facts, key)
        .and_then(|h| h.promises.as_deref())
        .is_some_and(|p| p.fundamental.is_some())
}
fn syntax(report: &Report, key: NodeKey) -> Option<ItemKind> {
    let unit = report.package.unit(key.file);
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return None;
    };
    let kind = unit.configured.ast.items[index as usize].kind;
    matches!(kind, ItemKind::Struct { .. } | ItemKind::Trait { .. }).then_some(kind)
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
    let error = |message| Error::at(key.file, attribute.span.start, message);
    if attribute.style != Style::Outer
        || attribute.input != Input::Empty
        || attribute.unsafe_wrapper
    {
        return Err(error("fundamental requires an outer word attribute"));
    }
    let unit = report.package.unit(key.file);
    let retained = unit.retained_attributes(effect..effect + 1).next();
    if retained.map(|(a, f)| (a, a.name(f, unit.source.text())))
        != Some((attribute, Some("fundamental")))
    {
        return Err(error("fundamental lost its actual retained attribute"));
    }
    if !report.package.requested_fundamental().map_err(error)? {
        let name = forest
            .tree(attribute.path.start)
            .ok_or_else(|| error("fundamental lost its name token"))?
            .span;
        let feature = FeatureName::new("fundamental").map_err(error)?;
        if !allowances.allows_at(key.file, name, &feature)? {
            return Err(error("fundamental attribute requires fundamental"));
        }
    }
    let syntax = syntax(report, key)
        .ok_or_else(|| error("fundamental requires an actual struct or trait"))?;
    if header(facts, key).is_none() {
        super::observe_selected_owner(report, facts, owner, key, true)?;
    }
    let actual = header(facts, key).ok_or_else(|| error("fundamental lost its source header"))?;
    super::charge_source_contract_query(facts, actual.owners.len())?;
    let owner_slot = actual
        .owners
        .iter()
        .position(|o| o.key == owner && o.effects.contains(&effect))
        .ok_or_else(|| error("fundamental belongs to another source owner"))?;
    if marked(facts, key) {
        return Err(error("duplicate fundamental attribute"));
    }
    let bytes = size_of::<Marker>()
        + if actual.promises.is_some() {
            0
        } else {
            size_of::<super::Promises>()
        };
    let actual = ledger(facts, bytes)?
        .local
        .declarations
        .get_mut(&key)
        .ok_or_else(|| error("fundamental source header disappeared"))?;
    actual
        .promises
        .get_or_insert_with(Default::default)
        .fundamental = Some(Box::new(Marker {
        owner_slot,
        effect,
        attribute,
        syntax,
        definition: None,
    }));
    Ok(())
}
pub(super) fn bind(
    report: &Report,
    facts: &mut Declarations,
    id: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    let actual = report
        .resolution
        .definitions
        .get(id.0 as usize)
        .ok_or_else(|| Error::unlocated("fundamental lost its definition"))?;
    if matches!(actual.kind, DefKind::Constructor | DefKind::SelfType) {
        return Ok(());
    }
    let Some(h) = header(facts, key) else {
        return Ok(());
    };
    let Some(marker) = h.promises.as_deref().and_then(|p| p.fundamental.as_deref()) else {
        return Ok(());
    };
    if actual.node != Some(key)
        || !matches!(
            (marker.syntax, actual.kind),
            (ItemKind::Struct { .. }, DefKind::Struct) | (ItemKind::Trait { .. }, DefKind::Trait)
        )
    {
        return Err(Error::at(
            key.file,
            marker.attribute.span.start,
            "fundamental changed its defining target",
        ));
    }
    validate(report, facts, key, h, marker)?;
    if marker.definition.is_some() {
        return Err(Error::at(
            key.file,
            0,
            "fundamental bound more than one defining identity",
        ));
    }
    let local = &mut ledger(facts, 0)?.local;
    local
        .declarations
        .get_mut(&key)
        .and_then(|h| h.promises.as_deref_mut())
        .and_then(|p| p.fundamental.as_deref_mut())
        .ok_or_else(|| Error::at(key.file, 0, "fundamental binding lost its marker"))?
        .definition = Some(id);
    local.first_fundamental.get_or_insert(id);
    Ok(())
}
fn validate(
    report: &Report,
    facts: &Declarations,
    key: NodeKey,
    h: &Header,
    m: &Marker,
) -> Result<SourceContractSite, Error> {
    super::validate_source(facts, report, key, h)?;
    let error = |message| Error::at(key.file, m.attribute.span.start, message);
    let owner = h
        .owners
        .get(m.owner_slot)
        .filter(|o| o.effects.contains(&m.effect))
        .ok_or_else(|| error("fundamental changed its configured effect owner"))?;
    let actual = report
        .package
        .unit(key.file)
        .retained_attributes(m.effect..m.effect + 1)
        .next();
    if actual.map(|(a, _)| a) != Some(m.attribute) || syntax(report, key) != Some(m.syntax) {
        return Err(error(
            "fundamental changed its source declaration or effect",
        ));
    }
    Ok(SourceContractSite {
        owner: owner.key,
        declaration: key,
        span: m.attribute.span,
    })
}
/// Internal typed consumers reuse the completed producer; they do not inspect ASTs.
pub(in crate::sema) fn policy(facts: &Declarations, id: DefId) -> Result<bool, &'static str> {
    if let Some(local) = facts
        .must_use_contracts
        .as_deref()
        .and_then(|o| o.headers.as_deref())
        .map(|h| &h.local)
        && let Some(key) = local.definitions.get(&id)
        && let Some(marker) = local
            .declarations
            .get(key)
            .and_then(|h| h.promises.as_deref())
            .and_then(|p| p.fundamental.as_deref())
    {
        super::charge_query(local, 4)
            .map_err(|_| "fundamental query exceeds source contract budget")?;
        if !local.comptime_complete || marker.definition != Some(id) {
            return Err("fundamental query requires its completed defining identity");
        }
        return Ok(true);
    }
    // Existing wire profiles encode this policy only for actual sizing traits.
    if let Some(catalog) = facts.source_bindings.as_deref()
        && let Some(role) = catalog.sizing_kind(id)?
    {
        let actual = catalog
            .validated_role(role)?
            .ok_or("fundamental sizing role disappeared")?;
        return Ok(catalog
            .neutral_attributes(actual)?
            .contains(crate::resolve::bindings::HeaderAttribute::Fundamental));
    }
    Ok(false)
}
impl Declarations {
    pub(crate) fn fundamental_contract(
        &self,
        report: &Report,
        id: DefId,
    ) -> Result<ContractKnowledge<SourceContractSite>, Error> {
        self.require_lint_context(report)?;
        let actual = report
            .resolution
            .definitions
            .get(id.0 as usize)
            .ok_or_else(|| Error::unlocated("fundamental query has no actual declaration"))?;
        let Some(key) = actual.node else {
            return Ok(ContractKnowledge::Unknown);
        };
        if !matches!(actual.kind, DefKind::Struct | DefKind::Trait) || syntax(report, key).is_none()
        {
            return Err(Error::at(
                key.file,
                0,
                "fundamental query requires a defining struct or trait",
            ));
        }
        let Some(local) = self
            .must_use_contracts
            .as_deref()
            .and_then(|o| o.headers.as_deref())
            .map(|h| &h.local)
        else {
            return Ok(ContractKnowledge::Unknown);
        };
        super::charge_query(local, 6)?;
        let Some(h) = local
            .declarations
            .get(&key)
            .filter(|_| local.comptime_complete && local.definitions.get(&id) == Some(&key))
        else {
            return Ok(ContractKnowledge::Unknown);
        };
        if let Some(m) = h.promises.as_deref().and_then(|p| p.fundamental.as_deref()) {
            if m.definition != Some(id) {
                return Err(Error::at(
                    key.file,
                    0,
                    "fundamental changed its bound definition",
                ));
            }
            Ok(ContractKnowledge::KnownPresent(validate(
                report, self, key, h, m,
            )?))
        } else {
            super::validate_source(self, report, key, h)?;
            Ok(ContractKnowledge::KnownAbsent)
        }
    }
    pub(crate) fn require_fundamental_transport(
        &self,
        report: &Report,
    ) -> Result<(), &'static str> {
        let Some(local) = self
            .must_use_contracts
            .as_deref()
            .and_then(|o| o.headers.as_deref())
            .map(|h| &h.local)
            .filter(|h| h.first_fundamental.is_some())
        else {
            return Ok(());
        };
        super::charge_query(local, local.definitions.len())
            .map_err(|_| "fundamental publication exceeds source contract budget")?;
        for (&id, key) in &local.definitions {
            let Some(marker) = local
                .declarations
                .get(key)
                .and_then(|h| h.promises.as_deref())
                .and_then(|p| p.fundamental.as_deref())
                .filter(|m| m.definition == Some(id))
            else {
                continue;
            };
            if !matches!(
                self.fundamental_contract(report, id)
                    .map_err(|_| "fundamental publication lost its source contract")?,
                ContractKnowledge::KnownPresent(_)
            ) {
                return Err("fundamental publication requires its completed source contract");
            }
            if matches!(marker.syntax, ItemKind::Trait { .. })
                && self
                    .source_bindings
                    .as_deref()
                    .map(|c| c.sizing_kind(id))
                    .transpose()?
                    .flatten()
                    .is_some()
            {
                continue;
            }
            return Err(
                "general fundamental policy requires authenticated trait/nominal metadata transport",
            );
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/fundamental_headers.rs"]
mod tests;
