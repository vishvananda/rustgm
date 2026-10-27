//! Written method policies retain their actual owner and defining body.
use super::{Checker, DefId, Error, NodeKey};
use crate::{sema::associated::OwnerKind, syntax::ast::NodeId};
pub(in crate::sema) fn is_final_trait_method(
    report: &crate::resolve::expansion::Report,
    facts: &crate::sema::model::Declarations,
    definition: DefId,
) -> Result<bool, Error> {
    let actual = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .ok_or_else(|| Error::unlocated("final method definition disappeared"))?;
    let Some(key) = actual.node else {
        if crate::sema::header_attributes::bootstrap_owner(report, definition)? {
            return Ok(false);
        }
        if let Some(catalog) = facts
            .crates
            .as_ref()
            .and_then(|linkage| linkage.origin_catalog())
            && let Some(imported) = catalog
                .declaration_if_imported(definition)
                .map_err(Error::unlocated)?
        {
            let header = facts
                .declaration_header(definition)
                .map_err(Error::unlocated)?
                .ok_or_else(|| Error::unlocated("imported method lost its declaration header"))?;
            if header.declaration
                != crate::resolve::body_origin::DeclarationOrigin::Imported(imported)
            {
                return Err(Error::unlocated(
                    "final policy changed its imported declaration",
                ));
            }
            // An authenticated required declaration cannot have a final body.
            // A provided body still needs its explicit transported policy.
            if header.body.is_none() {
                return Ok(false);
            }
            if crate::sema::header_attributes::has_plain_provided_function_policy(facts, definition)
                .map_err(Error::unlocated)?
            {
                return Ok(false);
            }
        }
        return Err(Error::unlocated(
            "imported or missing final trait method policy is not available",
        ));
    };
    let Some(modifier) = report.active_finalness(key)? else {
        return Ok(false);
    };
    let owner = facts
        .associated
        .owner(definition)
        .and_then(|id| facts.associated.get(id));
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|&index| facts.functions.get(index));
    if actual.kind != crate::resolve::model::DefKind::Function
        || owner.is_none_or(|owner| !matches!(owner.kind, OwnerKind::Trait { .. }))
        || function.is_none_or(|function| {
            function.definition != definition || function.node != key || function.body.is_none()
        })
    {
        return Err(Error::at(
            key.file,
            modifier.span.start,
            "final method lost its actual trait body",
        ));
    }
    Ok(true)
}
impl Checker<'_> {
    pub(super) fn validate_default_function(
        &self,
        key: NodeKey,
        definition: Option<DefId>,
        body: Option<NodeId>,
    ) -> Result<(), Error> {
        let Some(modifier) = self.report.active_method_modifier(key)? else {
            return Ok(());
        };
        if modifier.kind == crate::syntax::gate_sites::Kind::FinalFunction {
            let owner = definition
                .and_then(|id| self.facts.associated.owner(id))
                .and_then(|id| self.facts.associated.get(id));
            if body.is_none()
                || owner.is_none_or(|owner| !matches!(owner.kind, OwnerKind::Trait { .. }))
            {
                return Err(Error::at(
                    key.file,
                    modifier.span.start,
                    "final function requires its actual trait owner and defining body",
                ));
            }
            return Ok(());
        }
        let defaultness = modifier;
        if defaultness.node != key.node || body.is_none() {
            return Err(Error::at(
                key.file,
                defaultness.span.start,
                "default function requires its actual defining body",
            ));
        }
        let owner = definition
            .and_then(|id| self.facts.associated.owner(id))
            .and_then(|owner| self.facts.associated.get(owner));
        if owner.is_none_or(|owner| !matches!(owner.kind, OwnerKind::Impl { .. })) {
            return Err(Error::at(
                key.file,
                defaultness.span.start,
                "default function requires an actual impl owner",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/sealed_members.rs"]
mod tests;
