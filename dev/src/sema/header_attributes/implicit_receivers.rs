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

/// False means Unknown, including imported or incomplete headers. No caller may
/// infer checked absence from that outcome. One authenticated source walk is shared.
pub(super) fn visit_words(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
    mut visit: impl FnMut(Option<&str>),
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
            visit(attribute.name(forest, unit.source.text()));
        }
    }
    Ok(true)
}

impl Declarations {
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
