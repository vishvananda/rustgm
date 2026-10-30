//! The defining target owns Has; a method's Allow never creates that authority.
use super::{Declarations, DefId, Error, Header, NodeKey, Report, ledger};
use crate::{
    resolve::model::DefKind,
    sema::header_attributes::{ContractKnowledge, SourceContractSite},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::ItemKind,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Target {
    Trait {
        auto_: bool,
        unsafe_: bool,
        constant: bool,
    },
    Struct,
    Enum,
    Union,
    Foreign,
}
impl Target {
    fn of(report: &Report, key: NodeKey) -> Option<Self> {
        let unit = report.package.unit(key.file);
        let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
            return None;
        };
        Some(match unit.configured.ast.items[index as usize].kind {
            ItemKind::Trait {
                auto_,
                unsafe_,
                constant,
                ..
            } => Self::Trait {
                auto_,
                unsafe_,
                constant,
            },
            ItemKind::Struct { .. } => Self::Struct,
            ItemKind::Enum { .. } => Self::Enum,
            ItemKind::Union { .. } => Self::Union,
            ItemKind::ForeignType => Self::Foreign,
            _ => return None,
        })
    }
    fn matches(self, kind: DefKind) -> bool {
        matches!(
            (self, kind),
            (Self::Trait { .. }, DefKind::Trait)
                | (Self::Struct | Self::Union, DefKind::Struct)
                | (Self::Enum, DefKind::Enum)
                | (Self::Foreign, DefKind::ForeignType)
        )
    }
}
#[derive(Clone, Copy)]
pub(super) struct Marker {
    owner_slot: usize,
    effect: usize,
    attribute: Attribute,
    target: Target,
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
        .and_then(|row| row.promises.as_deref())
        .is_some_and(|row| row.incoherent.is_some())
}
pub(in crate::sema) fn consume(
    report: &Report,
    facts: &mut Declarations,
    (owner, key): (NodeKey, NodeKey),
    (attribute, effect): (Attribute, usize),
) -> Result<(), Error> {
    facts.require_lint_context(report)?;
    facts.reserve_must_use_source(key, attribute.span)?;
    let error = |message| Error::at(key.file, attribute.span.start, message);
    if attribute.style != Style::Outer
        || attribute.input != Input::Empty
        || attribute.unsafe_wrapper
    {
        return Err(error(
            "rustc_has_incoherent_inherent_impls requires an outer word attribute",
        ));
    }
    let unit = report.package.unit(key.file);
    let (actual, forest) = unit
        .retained_attributes(effect..effect + 1)
        .next()
        .ok_or_else(|| error("incoherent target lost its actual attribute"))?;
    if actual != attribute
        || actual.name(forest, unit.source.text()) != Some("rustc_has_incoherent_inherent_impls")
    {
        return Err(error(
            "incoherent target attribute changed its source effect",
        ));
    }
    crate::resolve::bindings::root_options::require_internal(
        report,
        &mut report.macro_allowances(),
        key.file,
        attribute,
        forest,
    )?;
    let target = Target::of(report, key)
        .ok_or_else(|| error("rustc_has_incoherent_inherent_impls requires a trait, struct, enum, union or foreign type"))?;
    if header(facts, key).is_none() {
        super::observe_selected_owner(report, facts, owner, key, true)?;
    }
    let actual =
        header(facts, key).ok_or_else(|| error("incoherent target lost its source header"))?;
    super::charge_source_contract_query(facts, actual.owners.len())?;
    let owner_slot = actual
        .owners
        .iter()
        .position(|row| row.key == owner && row.effects.contains(&effect))
        .ok_or_else(|| error("incoherent target attribute belongs to another owner"))?;
    if marked(facts, key) {
        return Err(error(
            "duplicate rustc_has_incoherent_inherent_impls attribute",
        ));
    }
    let bytes = if actual.promises.is_some() {
        0
    } else {
        size_of::<super::Promises>()
    };
    let headers = &mut ledger(facts, bytes)?.local;
    let actual = headers
        .declarations
        .get_mut(&key)
        .ok_or_else(|| error("incoherent target header disappeared"))?;
    actual
        .promises
        .get_or_insert_with(Default::default)
        .incoherent = Some(Marker {
        owner_slot,
        effect,
        attribute,
        target,
    });
    Ok(())
}
pub(super) fn bind(
    report: &Report,
    facts: &mut Declarations,
    definition: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    let Some(actual) = header(facts, key) else {
        return Ok(());
    };
    let Some(marker) = actual
        .promises
        .as_deref()
        .and_then(|row| row.incoherent.as_ref())
    else {
        return Ok(());
    };
    if report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .is_some_and(|row| matches!(row.kind, DefKind::Constructor | DefKind::SelfType))
    {
        return Ok(());
    }
    super::charge_source_contract_query(facts, 4)?;
    report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .filter(|row| row.node == Some(key) && marker.target.matches(row.kind))
        .ok_or_else(|| {
            Error::at(
                key.file,
                0,
                "incoherent target changed its actual definition",
            )
        })?;
    validate(report, facts, key, actual, marker)?;
    ledger(facts, 0)?
        .local
        .first_incoherent_target
        .get_or_insert(definition);
    Ok(())
}
fn validate(
    report: &Report,
    facts: &Declarations,
    key: NodeKey,
    actual: &Header,
    marker: &Marker,
) -> Result<SourceContractSite, Error> {
    super::validate_source(facts, report, key, actual)?;
    let error = |message| Error::at(key.file, marker.attribute.span.start, message);
    let owner = actual
        .owners
        .get(marker.owner_slot)
        .filter(|owner| owner.effects.contains(&marker.effect))
        .ok_or_else(|| error("incoherent target changed its configured effect owner"))?;
    let unit = report.package.unit(key.file);
    let retained = unit
        .retained_attributes(marker.effect..marker.effect + 1)
        .next()
        .ok_or_else(|| error("incoherent target effect disappeared"))?;
    if retained.0 != marker.attribute || Target::of(report, key) != Some(marker.target) {
        return Err(error(
            "incoherent target changed its source declaration or attribute",
        ));
    }
    Ok(SourceContractSite {
        owner: owner.key,
        declaration: key,
        span: marker.attribute.span,
    })
}
impl Declarations {
    /// Has belongs to this actual defining declaration. Legacy imports stay unknown.
    pub(crate) fn incoherent_inherent_contract(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<ContractKnowledge<SourceContractSite>, Error> {
        self.require_lint_context(report)?;
        let declared = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| Error::unlocated("incoherent target definition is absent"))?;
        let Some(key) = declared.node else {
            return Ok(ContractKnowledge::Unknown);
        };
        let target = Target::of(report, key)
            .filter(|target| target.matches(declared.kind))
            .ok_or_else(|| {
                Error::at(
                    key.file,
                    0,
                    "incoherent query requires an actual defining target",
                )
            })?;
        let Some(local) = self
            .must_use_contracts
            .as_deref()
            .and_then(|row| row.headers.as_deref())
            .map(|ledger| &ledger.local)
        else {
            return Ok(ContractKnowledge::Unknown);
        };
        super::charge_source_contract_query(self, 6)?;
        let Some(actual) = local.declarations.get(&key).filter(|_| {
            local.comptime_complete && local.definitions.get(&definition) == Some(&key)
        }) else {
            return Ok(ContractKnowledge::Unknown);
        };
        match actual
            .promises
            .as_deref()
            .and_then(|row| row.incoherent.as_ref())
        {
            Some(marker) if marker.target == target => Ok(ContractKnowledge::KnownPresent(
                validate(report, self, key, actual, marker)?,
            )),
            Some(_) => Err(Error::at(
                key.file,
                0,
                "incoherent target changed its recorded kind",
            )),
            None => {
                super::validate_source(self, report, key, actual)?;
                Ok(ContractKnowledge::KnownAbsent)
            }
        }
    }
    /// Existing wire profiles carry neither target Has nor an incoherent impl index.
    pub(crate) fn require_incoherent_target_transport(
        &self,
        report: &Report,
    ) -> Result<(), &'static str> {
        if let Some(definition) = self
            .must_use_contracts
            .as_deref()
            .and_then(|row| row.headers.as_deref())
            .and_then(|ledger| ledger.local.first_incoherent_target)
        {
            if !matches!(
                self.incoherent_inherent_contract(report, definition)
                    .map_err(
                        |_| "incoherent target transport lost its authenticated source declaration"
                    )?,
                ContractKnowledge::KnownPresent(_)
            ) {
                return Err("incoherent target transport lost its completed source contract");
            }
            return Err(
                "incoherent target authority and inherent index require their wider metadata profile",
            );
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/incoherent_authority.rs"]
mod tests;
