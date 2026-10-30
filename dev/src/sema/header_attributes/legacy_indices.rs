//! Local legacy-const declaration contracts do not rewrite local calls.
mod parsing;
use super::{Declarations, DefId, Error, Header, NodeKey, Report, ledger};
use crate::{
    resolve::{
        expansion::MacroAllowanceSession,
        model::{DefKind, Namespace},
    },
    syntax::{
        attribute::Attribute,
        item::{FunctionFlags, ItemKind},
        token_tree::Forest,
    },
    types::{TypeId, arena::Identity},
};

pub(super) struct Marker {
    owner_slot: usize,
    effect: usize,
    attribute: Attribute,
    flags: FunctionFlags,
    generics: crate::syntax::ast::List,
    parameters: crate::syntax::ast::List,
    formals: usize,
    values: usize,
    positions: Vec<usize>,
    bound: Option<DefId>,
    completed: Option<(DefId, TypeId, Identity)>,
}
#[cfg(test)]
#[path = "../../../tests/invariants/legacy_headers.rs"]
mod tests;
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
fn marker(facts: &Declarations, key: NodeKey) -> Option<&Marker> {
    header(facts, key)?.promises.as_deref()?.legacy.as_deref()
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
    let error = |message: &str| Error::at(key.file, attribute.span.start, message);
    let unit = report.package.unit(key.file);
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("legacy marker name disappeared"))?
        .span;
    if !report.package.requested_internal_attributes()
        && !allowances.allows_at(
            key.file,
            name,
            &crate::resolve::bindings::FeatureName::new("rustc_attrs").map_err(error)?,
        )?
    {
        return Err(error(
            "rustc_legacy_const_generics requires its rustc_attrs request",
        ));
    }
    if report.package.source_core().is_none() {
        super::observe_selected_owner(report, facts, owner, key, true)?;
    }
    let actual =
        header(facts, key).ok_or_else(|| error("legacy marker lacks actual source header"))?;
    super::charge_source_contract_query(facts, actual.owners.len().saturating_add(1))?;
    let slot = actual
        .owners
        .iter()
        .position(|row| row.key == owner && row.effects.contains(&effect))
        .ok_or_else(|| error("legacy marker belongs to another actual owner"))?;
    if actual
        .promises
        .as_deref()
        .is_some_and(|row| row.legacy.is_some())
    {
        return Err(error("duplicate rustc_legacy_const_generics attribute"));
    }
    let crate::syntax::ast::Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error("legacy marker requires its actual free function"));
    };
    let item = unit.configured.ast.items[index as usize];
    let ItemKind::Function {
        flags, parameters, ..
    } = item.kind
    else {
        return Err(error("legacy marker requires a free function"));
    };
    let positions = parsing::indices(report, facts, key, attribute, forest)?;
    let (formals, values) = parsing::shape(report, facts, key)?;
    if positions.len() != formals {
        return Err(error(
            "legacy index count must equal the const generic parameter count",
        ));
    }
    super::charge_source_contract_query(facts, positions.len())?;
    if positions
        .iter()
        .any(|index| *index >= formals.saturating_add(values))
    {
        return Err(error(
            "legacy const index exceeds the original call argument count",
        ));
    }
    let promises = header(facts, key)
        .ok_or_else(|| error("legacy header disappeared"))?
        .promises
        .is_some();
    let bytes = size_of::<Marker>()
        + if promises {
            0
        } else {
            size_of::<super::Promises>()
        };
    ledger(facts, bytes)?;
    let row = ledger(facts, 0)?
        .local
        .declarations
        .get_mut(&key)
        .ok_or_else(|| error("legacy header disappeared"))?;
    row.promises.get_or_insert_with(Default::default).legacy = Some(Box::new(Marker {
        owner_slot: slot,
        effect,
        attribute,
        flags,
        generics: item.generics,
        parameters,
        formals,
        values,
        positions,
        bound: None,
        completed: None,
    }));
    Ok(())
}
fn current<'a>(
    report: &Report,
    facts: &'a Declarations,
    key: NodeKey,
) -> Result<&'a Marker, Error> {
    let row =
        header(facts, key).ok_or_else(|| Error::unlocated("legacy source header disappeared"))?;
    super::validate_source(facts, report, key, row)?;
    let marker =
        marker(facts, key).ok_or_else(|| Error::unlocated("legacy source marker disappeared"))?;
    let owner = row
        .owners
        .get(marker.owner_slot)
        .ok_or_else(|| Error::unlocated("legacy owner slot changed"))?;
    let unit = report.package.unit(key.file);
    let actual = unit
        .retained_attributes(marker.effect..marker.effect + 1)
        .next()
        .ok_or_else(|| Error::unlocated("legacy effect disappeared"))?;
    let crate::syntax::ast::Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(Error::unlocated("legacy item changed"));
    };
    if !owner.effects.contains(&marker.effect)
        || actual.0 != marker.attribute
        || actual.0.name(actual.1, unit.source.text()) != Some("rustc_legacy_const_generics")
        || unit.configured.ast.items[index as usize].generics != marker.generics
        || !matches!(unit.configured.ast.items[index as usize].kind,ItemKind::Function{flags,parameters,..} if flags==marker.flags && parameters==marker.parameters)
    {
        return Err(Error::unlocated(
            "legacy actual source function or attribute changed",
        ));
    }
    Ok(marker)
}
fn free_function(
    report: &Report,
    facts: &Declarations,
    id: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    let actual = report
        .resolution
        .definitions
        .get(id.0 as usize)
        .ok_or_else(|| Error::unlocated("legacy function identity absent"))?;
    if actual.kind != DefKind::Function
        || actual.namespace != Namespace::Value
        || actual.node != Some(key)
        || report.resolution.foreign_owners.contains_key(&id)
        || facts.associated.owner(id).is_some()
    {
        return Err(Error::unlocated(
            "rustc_legacy_const_generics requires a free non-foreign function",
        ));
    }
    Ok(())
}
pub(super) fn bind(
    report: &Report,
    facts: &mut Declarations,
    id: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    if marker(facts, key).is_none() {
        return Ok(());
    }
    facts.require_lint_context(report)?;
    super::charge_source_contract_query(facts, 3)?;
    current(report, facts, key)?;
    free_function(report, facts, id, key)?;
    let row = ledger(facts, 0)?
        .local
        .declarations
        .get_mut(&key)
        .and_then(|r| r.promises.as_deref_mut())
        .and_then(|r| r.legacy.as_deref_mut())
        .ok_or_else(|| Error::unlocated("legacy source owner disappeared"))?;
    row.bound = Some(id);
    Ok(())
}
impl Declarations {
    pub(in crate::sema) fn validate_legacy_indices_function(
        &mut self,
        report: &Report,
        function: &crate::sema::model::Function,
    ) -> Result<(), Error> {
        let key = function.node;
        if marker(self, key).is_none() {
            return Ok(());
        }
        self.require_lint_context(report)?;
        super::charge_source_contract_query(self, 4)?;
        let marker = current(report, self, key)?;
        if marker.bound != Some(function.definition) {
            return Err(Error::unlocated(
                "legacy function lost its actual bound declaration",
            ));
        }
        let parameters = self
            .generics
            .parameters
            .get(&function.definition)
            .map_or(&[][..], Vec::as_slice);
        super::charge_source_contract_query(self, parameters.len())?;
        if parameters.len() != marker.formals {
            return Err(Error::unlocated(
                "legacy typed const formals differ from declaration",
            ));
        }
        for parameter in parameters {
            let crate::types::arguments::Argument::Const(id) = parameter.argument else {
                return Err(Error::unlocated(
                    "legacy typed formal is not a const parameter",
                ));
            };
            let Some(crate::types::arguments::ConstKind::Parameter(id)) = self.types.const_kind(id)
            else {
                return Err(Error::unlocated(
                    "legacy formal is not the actual rigid const parameter",
                ));
            };
            if self.types.parameter(id).is_none_or(|p| {
                p.owner != crate::types::ItemId(function.definition.0)
                    || !matches!(
                        p.kind,
                        crate::types::parameters::ParameterKind::Const { .. }
                    )
            }) {
                return Err(Error::unlocated(
                    "legacy const formal belongs to another owner",
                ));
            }
        }
        let Some(crate::types::Kind::Function {
            bound_regions,
            parameters,
            ..
        }) = self.types.kind(function.signature)
        else {
            return Err(Error::unlocated(
                "legacy function lacks its actual signature",
            ));
        };
        if bound_regions != 0 {
            return Err(Error::unlocated(
                "rustc_legacy_const_generics permits only const generics, not outer lifetime parameters",
            ));
        }
        if self
            .types
            .list(parameters)
            .is_none_or(|types| types.len() != marker.values)
        {
            return Err(Error::unlocated(
                "legacy completed argument count differs from declaration",
            ));
        }
        let growth = if self.types.existing_arena_identity().is_none() {
            Identity::allocation_bytes()
        } else {
            0
        };
        super::reserve_source_contracts(self, 1, growth)?;
        let identity = self.types.arena_identity();
        let row = ledger(self, 0)?
            .local
            .declarations
            .get_mut(&key)
            .and_then(|r| r.promises.as_deref_mut())
            .and_then(|r| r.legacy.as_deref_mut())
            .ok_or_else(|| Error::unlocated("legacy source marker disappeared"))?;
        row.completed = Some((function.definition, function.signature, identity));
        Ok(())
    }
    pub(crate) fn reject_legacy_indices_transport(
        &self,
        report: &Report,
        id: DefId,
    ) -> Result<(), Error> {
        let Some(key) = report
            .resolution
            .definitions
            .get(id.0 as usize)
            .and_then(|d| d.node)
        else {
            return Ok(());
        };
        if marker(self, key).is_none() {
            return Ok(());
        }
        self.legacy_const_positions(report, id)?;
        Err(Error::unlocated(
            "legacy const generic indices require their cross-crate transport and argument protocol",
        ))
    }
    pub(crate) fn legacy_const_positions(
        &self,
        report: &Report,
        id: DefId,
    ) -> Result<&[usize], Error> {
        self.require_lint_context(report)?;
        super::charge_source_contract_query(self, 5)?;
        let key = report
            .resolution
            .definitions
            .get(id.0 as usize)
            .and_then(|d| d.node)
            .ok_or_else(|| Error::unlocated("legacy query requires its local source function"))?;
        free_function(report, self, id, key)?;
        let row = header(self, key)
            .ok_or_else(|| Error::unlocated("legacy source header disappeared"))?;
        super::validate_header(report, self, id, key, row)?;
        let marker = current(report, self, key)?;
        let Some((definition, signature, arena)) = &marker.completed else {
            return Err(Error::unlocated(
                "legacy completed function contract is pending",
            ));
        };
        let actual = self
            .function_indices
            .get(&id)
            .and_then(|index| self.functions.get(*index));
        if *definition != id
            || marker.bound != Some(id)
            || !self.types.has_identity(arena)
            || actual.is_none_or(|f| {
                f.definition != id
                    || f.node != key
                    || f.signature != *signature
                    || f.parameters != marker.parameters
            })
        {
            return Err(Error::unlocated(
                "legacy function signature, source owner or arena changed",
            ));
        }
        Ok(&marker.positions)
    }
}
