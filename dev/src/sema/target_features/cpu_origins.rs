//! Current checked owners authenticate policy; missing imported policy never weakens safety.
use super::{Contract, Declarations, DefId, Error, NodeKey, Policy, Report};
use crate::{
    sema::associated::OwnerKind,
    syntax::{
        ast::Kind,
        item::{ItemKind, Safety},
    },
};
impl Contract {
    pub(super) fn current(
        &self,
        report: &Report,
        facts: &Declarations,
        source: NodeKey,
    ) -> Result<(), Error> {
        let error = |message| {
            Error::at(
                source.file,
                self.sites
                    .first()
                    .map_or(0, |site| site.attribute.span.start),
                message,
            )
        };
        super::header_attributes::charge_source_contract_query(facts, 1 + self.sites.len())?;
        let unit = report
            .package
            .units
            .get(source.file.0 as usize)
            .ok_or_else(|| error("target-feature source file changed"))?;
        if source != self.source || !self.image.current(unit, source.node) || self.sites.is_empty()
        {
            return Err(error("target-feature source incarnation changed"));
        }
        let Kind::Item(index) = unit.configured.ast.node(source.node).kind else {
            return Err(error("target-feature source is not a function"));
        };
        let ItemKind::Function { flags, .. } = unit.configured.ast.items[index as usize].kind
        else {
            return Err(error("target-feature function kind changed"));
        };
        if (flags.safety == Safety::Unsafe) != self.declared_unsafe {
            return Err(error("target-feature declared safety changed"));
        }
        let mut prior = None;
        for site in &self.sites {
            super::header_attributes::charge_source_contract_query(facts, 1 + site.input.len())?;
            let effect = unit
                .configured
                .effects
                .get(site.effect)
                .ok_or_else(|| error("target-feature retained effect disappeared"))?;
            if effect.capture != site.capture || effect.attribute.attribute != site.attribute {
                return Err(error("target-feature retained input identity changed"));
            }
            let forest = &unit
                .configured
                .ast
                .attributes
                .get(site.capture as usize)
                .ok_or_else(|| error("target-feature capture disappeared"))?
                .forest;
            if site
                .input
                .iter()
                .any(|(index, tree)| forest.tree(*index) != Some(*tree))
            {
                return Err(error("target-feature input trees changed"));
            }
            if prior == Some(site.owner) {
                continue;
            }
            report
                .lint_producer()
                .ok_or_else(|| error("target-feature source owner lacks its producer"))?
                .validate_owner(&report.package, site.owner)
                .map_err(Error::from)?;
            let mut node = site.owner.node;
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                super::header_attributes::charge_source_contract_query(facts, 1)?;
                node = value;
            }
            if site.owner.file != source.file || node != source.node {
                return Err(error(
                    "target-feature configured owner changed its function",
                ));
            }
            prior = Some(site.owner);
        }
        Ok(())
    }
}
pub(in crate::sema) fn validate_method(
    facts: &Declarations,
    definition: DefId,
) -> Result<(), Error> {
    let Some(row) = facts.target_feature_functions.get(&definition) else {
        return Ok(());
    };
    if row.declared_unsafe {
        return Ok(());
    }
    if facts
        .associated
        .owner(definition)
        .and_then(|owner| facts.associated.get(owner))
        .is_some_and(|owner| {
            matches!(
                owner.kind,
                OwnerKind::Trait { .. }
                    | OwnerKind::Impl {
                        trait_type: Some(_),
                        ..
                    }
            )
        })
    {
        return Err(Error::at(
            row.source.file,
            row.sites
                .first()
                .map_or(0, |site| site.attribute.span.start),
            "target_feature cannot be applied to a safe trait method",
        ));
    }
    Ok(())
}
pub(in crate::sema) fn body_unsafe(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    signature_unsafe: bool,
) -> Result<bool, Error> {
    if !facts.target_feature_functions.contains_key(&definition) {
        return Ok(signature_unsafe);
    }
    match super::query(report, facts, definition)? {
        Policy::Local(Some(row)) => Ok(row.declared_unsafe),
        Policy::Local(None) | Policy::ImportedUnknown => Ok(signature_unsafe),
    }
}
