//! Definition owners share authenticated witnesses for distinct semantic policies.
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Policy {
    Fundamental,
    ForcedDyn,
}
impl Policy {
    const ALL: [Self; 2] = [Self::Fundamental, Self::ForcedDyn];
    const fn slot(self) -> usize {
        self as usize
    }
    fn named(name: &str) -> Option<Self> {
        match name {
            "fundamental" => Some(Self::Fundamental),
            "rustc_dyn_incompatible_trait" => Some(Self::ForcedDyn),
            _ => None,
        }
    }
    fn admits(self, kind: ItemKind) -> bool {
        matches!(kind, ItemKind::Trait { .. })
            || self == Self::Fundamental && matches!(kind, ItemKind::Struct { .. })
    }
    const fn transport_error(self) -> &'static str {
        match self {
            Self::Fundamental => {
                "general fundamental policy requires authenticated trait/nominal metadata transport"
            }
            Self::ForcedDyn => "forced dyn policy requires authenticated trait metadata transport",
        }
    }
}
struct Annotation {
    owner_slot: usize,
    effect: usize,
    attribute: Attribute,
}
pub(super) struct Marker {
    syntax: ItemKind,
    definition: Option<DefId>,
    annotations: [Option<Annotation>; 2],
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
fn marker(header: &Header) -> Option<&Marker> {
    header.promises.as_deref()?.policy.as_deref()
}
pub(super) fn marked(facts: &Declarations, key: NodeKey) -> bool {
    header(facts, key).and_then(marker).is_some()
}
fn syntax(report: &Report, key: NodeKey, policy: Policy) -> Option<ItemKind> {
    let unit = report.package.unit(key.file);
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return None;
    };
    let kind = unit.configured.ast.items[index as usize].kind;
    policy.admits(kind).then_some(kind)
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
    let unit = report.package.unit(key.file);
    let policy = attribute
        .name(forest, unit.source.text())
        .and_then(Policy::named)
        .ok_or_else(|| error("unknown declaration policy"))?;
    if attribute.style != Style::Outer
        || attribute.input != Input::Empty
        || attribute.unsafe_wrapper
    {
        return Err(error("declaration policy requires an outer word attribute"));
    }
    let retained = unit.retained_attributes(effect..effect + 1).next();
    if retained.map(|(a, f)| (a, a.name(f, unit.source.text()).and_then(Policy::named)))
        != Some((attribute, Some(policy)))
    {
        return Err(error("declaration policy lost its retained attribute"));
    }
    require_feature(report, allowances, key, attribute, forest, policy)?;
    let syntax = syntax(report, key, policy)
        .ok_or_else(|| error("declaration policy has the wrong actual struct/trait target"))?;
    if header(facts, key).is_none() {
        super::observe_selected_owner(report, facts, owner, key, true)?;
    }
    let actual =
        header(facts, key).ok_or_else(|| error("declaration policy lost its source header"))?;
    super::charge_source_contract_query(facts, actual.owners.len())?;
    let owner_slot = actual
        .owners
        .iter()
        .position(|o| o.key == owner && o.effects.contains(&effect))
        .ok_or_else(|| error("declaration policy belongs to another source owner"))?;
    if marker(actual).is_some_and(|m| m.annotations[policy.slot()].is_some()) {
        return Err(error("duplicate declaration policy attribute"));
    }
    let bytes = if marker(actual).is_some() {
        0
    } else {
        size_of::<Marker>()
    } + if actual.promises.is_some() {
        0
    } else {
        size_of::<super::Promises>()
    };
    let actual = ledger(facts, bytes)?
        .local
        .declarations
        .get_mut(&key)
        .ok_or_else(|| error("declaration policy source header disappeared"))?;
    let marker = actual
        .promises
        .get_or_insert_with(Default::default)
        .policy
        .get_or_insert_with(|| {
            Box::new(Marker {
                syntax,
                definition: None,
                annotations: [None, None],
            })
        });
    marker.annotations[policy.slot()] = Some(Annotation {
        owner_slot,
        effect,
        attribute,
    });
    Ok(())
}
fn require_feature(
    report: &Report,
    allowances: &mut MacroAllowanceSession<'_>,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    policy: Policy,
) -> Result<(), Error> {
    if policy == Policy::ForcedDyn {
        return crate::resolve::bindings::root_options::require_internal(
            report, allowances, key.file, attribute, forest,
        );
    }
    let error = |message| Error::at(key.file, attribute.span.start, message);
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
        .ok_or_else(|| Error::unlocated("declaration policy lost its definition"))?;
    if matches!(actual.kind, DefKind::Constructor | DefKind::SelfType) {
        return Ok(());
    }
    let Some(h) = header(facts, key) else {
        return Ok(());
    };
    let Some(m) = marker(h) else {
        return Ok(());
    };
    if actual.node != Some(key)
        || !matches!(
            (m.syntax, actual.kind),
            (ItemKind::Struct { .. }, DefKind::Struct) | (ItemKind::Trait { .. }, DefKind::Trait)
        )
    {
        return Err(Error::at(
            key.file,
            0,
            "declaration policy changed its defining target",
        ));
    }
    super::validate_source(facts, report, key, h)?;
    for p in Policy::ALL {
        if let Some(a) = &m.annotations[p.slot()] {
            validate(report, key, h, m, p, a)?;
        }
    }
    if m.definition.is_some() {
        return Err(Error::at(
            key.file,
            0,
            "declaration policy bound more than one defining identity",
        ));
    }
    let local = &mut ledger(facts, 0)?.local;
    local
        .declarations
        .get_mut(&key)
        .and_then(|h| h.promises.as_deref_mut())
        .and_then(|p| p.policy.as_deref_mut())
        .ok_or_else(|| Error::at(key.file, 0, "declaration policy binding lost its marker"))?
        .definition = Some(id);
    local.first_policy.get_or_insert(id);
    Ok(())
}
fn validate(
    report: &Report,
    key: NodeKey,
    h: &Header,
    m: &Marker,
    policy: Policy,
    a: &Annotation,
) -> Result<SourceContractSite, Error> {
    let error = |message| Error::at(key.file, a.attribute.span.start, message);
    let owner = h
        .owners
        .get(a.owner_slot)
        .filter(|o| o.effects.contains(&a.effect))
        .ok_or_else(|| error("declaration policy changed its configured effect owner"))?;
    let actual = report
        .package
        .unit(key.file)
        .retained_attributes(a.effect..a.effect + 1)
        .next();
    if actual.map(|(a, _)| a) != Some(a.attribute) || syntax(report, key, policy) != Some(m.syntax)
    {
        return Err(error(
            "declaration policy changed its source declaration or effect",
        ));
    }
    Ok(SourceContractSite {
        owner: owner.key,
        declaration: key,
        span: a.attribute.span,
    })
}
/// Internal typed consumers reuse completed producer facts without AST rescans.
fn read(facts: &Declarations, id: DefId, policy: Policy) -> Result<bool, &'static str> {
    let local = facts
        .must_use_contracts
        .as_deref()
        .and_then(|o| o.headers.as_deref())
        .map(|h| &h.local);
    if let Some(local) = local.filter(|h| h.first_policy.is_some()) {
        super::charge_query(local, 4)
            .map_err(|_| "declaration policy query exceeds source contract budget")?;
    }
    if let Some(local) = local
        && let Some(key) = local.definitions.get(&id)
        && let Some(m) = local.declarations.get(key).and_then(marker)
        && m.annotations[policy.slot()].is_some()
    {
        if !local.comptime_complete || m.definition != Some(id) {
            return Err("declaration policy query requires its completed defining identity");
        }
        return Ok(true);
    }
    if let Some(catalog) = facts.source_bindings.as_deref() {
        if policy == Policy::ForcedDyn {
            return catalog.forces_dyn_incompatible(id);
        }
        if let Some(role) = catalog.sizing_kind(id)? {
            let actual = catalog
                .validated_role(role)?
                .ok_or("fundamental sizing role disappeared")?;
            return Ok(catalog
                .neutral_attributes(actual)?
                .contains(crate::resolve::bindings::HeaderAttribute::Fundamental));
        }
    }
    Ok(false)
}
pub(in crate::sema) fn fundamental_policy(
    facts: &Declarations,
    id: DefId,
) -> Result<bool, &'static str> {
    read(facts, id, Policy::Fundamental)
}
pub(in crate::sema) fn forced_dyn_policy(
    facts: &Declarations,
    id: DefId,
) -> Result<bool, &'static str> {
    read(facts, id, Policy::ForcedDyn)
}
fn contract(
    facts: &Declarations,
    report: &Report,
    id: DefId,
    policy: Policy,
) -> Result<ContractKnowledge<SourceContractSite>, Error> {
    facts.require_lint_context(report)?;
    let actual = report
        .resolution
        .definitions
        .get(id.0 as usize)
        .ok_or_else(|| Error::unlocated("declaration policy query has no actual declaration"))?;
    let Some(key) = actual.node else {
        return Ok(ContractKnowledge::Unknown);
    };
    if !matches!(actual.kind, DefKind::Struct | DefKind::Trait)
        || syntax(report, key, policy).is_none()
    {
        return Err(Error::at(
            key.file,
            0,
            "declaration policy query has the wrong defining target",
        ));
    }
    let Some(local) = facts
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
    super::validate_source(facts, report, key, h)?;
    if let Some((m, a)) =
        marker(h).and_then(|m| m.annotations[policy.slot()].as_ref().map(|a| (m, a)))
    {
        if m.definition != Some(id) {
            return Err(Error::at(
                key.file,
                0,
                "declaration policy changed its bound definition",
            ));
        }
        Ok(ContractKnowledge::KnownPresent(validate(
            report, key, h, m, policy, a,
        )?))
    } else {
        Ok(ContractKnowledge::KnownAbsent)
    }
}
impl Declarations {
    pub(crate) fn fundamental_contract(
        &self,
        report: &Report,
        id: DefId,
    ) -> Result<ContractKnowledge<SourceContractSite>, Error> {
        contract(self, report, id, Policy::Fundamental)
    }
    pub(crate) fn dyn_restriction_contract(
        &self,
        report: &Report,
        id: DefId,
    ) -> Result<ContractKnowledge<SourceContractSite>, Error> {
        contract(self, report, id, Policy::ForcedDyn)
    }
    pub(crate) fn require_declaration_policy_transport(
        &self,
        report: &Report,
    ) -> Result<(), &'static str> {
        let Some(local) = self
            .must_use_contracts
            .as_deref()
            .and_then(|o| o.headers.as_deref())
            .map(|h| &h.local)
            .filter(|h| h.first_policy.is_some())
        else {
            return Ok(());
        };
        super::charge_query(local, local.definitions.len())
            .map_err(|_| "declaration policy publication exceeds source contract budget")?;
        for (&id, key) in &local.definitions {
            let Some(m) = local
                .declarations
                .get(key)
                .and_then(marker)
                .filter(|m| m.definition == Some(id))
            else {
                continue;
            };
            for policy in Policy::ALL {
                if m.annotations[policy.slot()].is_none() {
                    continue;
                }
                let known = match policy {
                    Policy::Fundamental => self.fundamental_contract(report, id),
                    Policy::ForcedDyn => self.dyn_restriction_contract(report, id),
                }
                .map_err(|_| "declaration policy publication lost its source contract")?;
                if !matches!(known, ContractKnowledge::KnownPresent(_)) {
                    return Err(
                        "declaration policy publication requires its completed source contract",
                    );
                }
                let sizing = self
                    .source_bindings
                    .as_deref()
                    .map(|c| c.sizing_kind(id))
                    .transpose()?
                    .flatten();
                if matches!(m.syntax, ItemKind::Trait { .. })
                    && sizing.is_some()
                    && (policy == Policy::Fundamental
                        || sizing == Some(crate::support::bindings::LangItem::Sized))
                {
                    continue;
                }
                return Err(policy.transport_error());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/forced_dyn_headers.rs"]
mod dyn_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/fundamental_headers.rs"]
mod tests;
