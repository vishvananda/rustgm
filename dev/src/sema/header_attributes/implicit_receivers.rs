//! Retained lint metadata never grants a reference, method or call permission.
use super::{Declarations, DefId, DefKind, Error, NodeKey, Report, charge_query, validate_header};
use crate::{
    sema::header_attributes::ContractKnowledge,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
    },
};

pub(in crate::sema) fn consume_no_implicit_autorefs(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
    forest: &crate::syntax::token_tree::Forest,
    allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error(
            "rustc_no_implicit_autorefs requires a word attribute",
        ));
    }
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("raw autoref marker lost its actual name"))?
        .span;
    if report.package.source_core().is_none()
        || (!report.package.requested_internal_attributes()
            && !allowances.allows_at(
                declaration.file,
                name,
                &crate::resolve::bindings::FeatureName::new("rustc_attrs").map_err(error)?,
            )?)
    {
        return Err(error(
            "raw autoref marker requires its rustc_attrs source profile",
        ));
    }
    let ast = &report.package.unit(declaration.file).configured.ast;
    if !matches!(ast.node(declaration.node).kind, Kind::Item(index)
        if matches!(ast.items[index as usize].kind, ItemKind::Function { .. }))
    {
        return Err(error(
            "raw autoref marker requires an actual function or method",
        ));
    }
    Ok(())
}

/// Pinned `rustc_as_ptr` is diagnostic data. It does not prove a pointer result,
/// reference validity, effects, const eligibility or an optimization contract.
pub(in crate::sema) fn consume_pointer_lint(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
    forest: &crate::syntax::token_tree::Forest,
    allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error("rustc_as_ptr requires a word attribute"));
    }
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("pointer lint marker lost its actual name"))?
        .span;
    if !report.package.requested_internal_attributes()
        && !allowances.allows_at(
            declaration.file,
            name,
            &crate::resolve::bindings::FeatureName::new("rustc_attrs").map_err(error)?,
        )?
    {
        return Err(error("rustc_as_ptr requires its rustc_attrs request"));
    }
    let ast = &report.package.unit(declaration.file).configured.ast;
    if !matches!(ast.node(declaration.node).kind, Kind::Item(index)
        if matches!(ast.items[index as usize].kind, ItemKind::Function { .. }))
    {
        return Err(error(
            "rustc_as_ptr requires its actual function or method owner",
        ));
    }
    Ok(())
}

/// The source marker describes an internal compiler lint and grants no call permission.
pub(in crate::sema) fn consume_query_instability(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
    forest: &crate::syntax::token_tree::Forest,
    allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error(
            "rustc_lint_query_instability requires a word attribute",
        ));
    }
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("query-instability marker lost its actual name"))?
        .span;
    if report.package.source_core().is_none()
        || (!report.package.requested_internal_attributes()
            && !allowances.allows_at(
                declaration.file,
                name,
                &crate::resolve::bindings::FeatureName::new("rustc_attrs").map_err(error)?,
            )?)
    {
        return Err(error(
            "rustc_lint_query_instability requires its rustc_attrs source profile",
        ));
    }
    let ast = &report.package.unit(declaration.file).configured.ast;
    if !matches!(ast.node(declaration.node).kind, Kind::Item(index)
        if matches!(ast.items[index as usize].kind, ItemKind::Function { .. }))
    {
        return Err(error(
            "rustc_lint_query_instability requires an actual function or method",
        ));
    }
    Ok(())
}

/// False means Unknown, including imported or incomplete headers. No caller may
/// infer checked absence from that outcome. One authenticated source walk is shared.
pub(super) fn visit_attributes(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
    mut visit: impl FnMut(
        Attribute,
        &crate::syntax::token_tree::Forest,
        &crate::resolve::unit::Unit,
    ) -> Result<(), Error>,
) -> Result<bool, Error> {
    let actual = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .filter(|row| row.kind == DefKind::Function)
        .ok_or_else(|| Error::unlocated("source words require an actual function"))?;
    let Some(key) = actual.node else {
        return Ok(false);
    };
    let Some(headers) = facts
        .must_use_contracts
        .as_deref()
        .and_then(|contracts| contracts.headers.as_deref())
        .map(|ledger| &ledger.local)
        .filter(|headers| headers.complete)
    else {
        return Ok(false);
    };
    charge_query(headers, 1)?;
    let header = headers
        .definitions
        .get(&definition)
        .filter(|&&actual| actual == key)
        .and_then(|_| headers.declarations.get(&key))
        .ok_or_else(|| Error::unlocated("source words changed their declaration identity"))?;
    validate_header(report, facts, definition, key, header)?;
    let unit = report.package.unit(key.file);
    for owner in &header.owners {
        charge_query(headers, owner.effects.len())?;
        for (attribute, forest) in unit.retained_attributes(owner.effects.clone()) {
            visit(attribute, forest, unit)?;
        }
    }
    Ok(true)
}

pub(super) fn visit_words(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
    mut visit: impl FnMut(Option<&str>),
) -> Result<bool, Error> {
    visit_attributes(facts, report, definition, |attribute, forest, unit| {
        visit(attribute.name(forest, unit.source.text()));
        Ok(())
    })
}

impl Declarations {
    /// Returns the actual function's diagnostic marker.
    /// # Errors
    /// Rejects changed source headers and mismatched imported declarations.
    pub fn pointer_lint_marker(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<ContractKnowledge<()>, Error> {
        if crate::sema::header_attributes::has_imported(self, definition) {
            return Ok(self
                .header_attribute_contracts(report, definition)?
                .imported_pointer_lint_marker);
        }
        let mut present = false;
        if !visit_words(self, report, definition, |name| {
            present |= name == Some("rustc_as_ptr");
        })? {
            return Ok(ContractKnowledge::Unknown);
        }
        Ok(if present {
            ContractKnowledge::KnownPresent(())
        } else {
            ContractKnowledge::KnownAbsent
        })
    }
    /// The boundary of every profile without the version-thirteen cold bit.
    pub(crate) fn reject_function_advisory_transport(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<(), Error> {
        self.publication_source_advisories(report, definition, false, false)
            .map(|_| ())
    }
    /// Match written callable advisories to the actual checked header.
    pub(crate) fn publication_source_advisories(
        &self,
        report: &Report,
        definition: DefId,
        cold_rows: bool,
        declaration_policies: bool,
    ) -> Result<(bool, &[usize]), Error> {
        let actual = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| {
                Error::unlocated("function advisory publication lost its declaration")
            })?;
        if actual.kind != DefKind::Function {
            return Ok((false, &[]));
        }
        if !declaration_policies
            && matches!(
                self.source_comptime_contract(report, definition)?,
                ContractKnowledge::KnownPresent(_)
            )
        {
            return Err(Error::unlocated(
                "always-comptime source contracts require a wider transport profile",
            ));
        }
        if !declaration_policies
            && self
                .associated
                .trait_owner(definition)
                .is_some_and(|parent| self.associated.constant_trait(parent) == Some(true))
            && matches!(
                self.source_non_const_trait_method_contract(report, definition)?,
                ContractKnowledge::KnownPresent(_)
            )
        {
            return Err(Error::unlocated(
                "non-const trait member restrictions require a wider transport profile",
            ));
        }
        let mut required = None;
        let mut pointer_lint_marker = false;
        let mut written_legacy = false;
        if !visit_words(self, report, definition, |name| {
            pointer_lint_marker |= name == Some("rustc_as_ptr");
            written_legacy |= name == Some("rustc_legacy_const_generics");
            let message = match name {
                Some("rustc_as_ptr") if !declaration_policies => {
                    Some("source rustc_as_ptr diagnostic marker transport is not implemented")
                }
                Some("cold") if !cold_rows => {
                    Some("source cold advisory marker transport is not implemented")
                }
                _ => None,
            };
            if required.is_none() {
                required = message;
            }
        })? {
            return Err(Error::unlocated(
                "function advisory publication requires completed actual source headers",
            ));
        }
        if let Some(message) = required {
            return Err(Error::unlocated(message));
        }
        let positions =
            self.legacy_const_positions_for_publication(report, definition, declaration_policies)?;
        if written_legacy == positions.is_empty() {
            return Err(Error::unlocated(
                "written legacy const positions lack the actual checked header",
            ));
        }
        Ok((pointer_lint_marker, positions))
    }

    /// The written cold advisory of a completed current function header. It
    /// carries no behavioral claim; consumers retain it as a header fact.
    pub(crate) fn cold_advisory(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<ContractKnowledge<()>, Error> {
        let mut present = false;
        if !visit_words(self, report, definition, |name| {
            present |= name == Some("cold");
        })? {
            return Ok(ContractKnowledge::Unknown);
        }
        Ok(if present {
            ContractKnowledge::KnownPresent(())
        } else {
            ContractKnowledge::KnownAbsent
        })
    }

    pub(in crate::sema) fn no_implicit_autorefs(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<ContractKnowledge<()>, Error> {
        let mut present = false;
        if !visit_words(self, report, definition, |name| {
            present |= name == Some("rustc_no_implicit_autorefs");
        })? {
            return Ok(ContractKnowledge::Unknown);
        }
        Ok(if present {
            ContractKnowledge::KnownPresent(())
        } else {
            ContractKnowledge::KnownAbsent
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/asptr_annotations.rs"]
mod tests;
