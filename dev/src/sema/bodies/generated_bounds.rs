//! Genuine generated endpoints use the ordinary member inference and execution.
use super::{Checker, DefId, Error, NodeId, TermId, TypeId};
use crate::sema::compiler_items::{RangeHelper, SourceRangeMember};
use crate::{
    resolve::model::Target,
    syntax::gate_sites::Helper,
    types::{Kind, infer::arguments::Argument, parameters::ParameterId},
};
#[cfg(test)]
#[path = "../../../tests/invariants/endpoint_execution.rs"]
mod tests;

impl Checker<'_> {
    pub(super) fn pattern_endpoint(&mut self, node: NodeId) -> Result<TermId, Error> {
        let key = self.key(node);
        let site = self.report.pattern_endpoint(key)?;
        let base = self
            .facts
            .node(site.base())
            .and_then(|n| n.ty)
            .ok_or_else(|| self.error(node, "pattern endpoint base annotation is incomplete"))?;
        let helper = match site.helper() {
            Helper::Minimum => RangeHelper::Minimum,
            Helper::Maximum => RangeHelper::Maximum,
            Helper::Predecessor => RangeHelper::Predecessor,
        };
        let member =
            crate::sema::compiler_items::source_range_member(self.report, self.facts, key, helper)?;
        let (parent, owner, term) = match member {
            SourceRangeMember::Bound {
                parent,
                member,
                self_type,
            } => {
                let ty = self
                    .facts
                    .definitions
                    .get(member.0 as usize)
                    .copied()
                    .flatten()
                    .ok_or_else(|| self.error(node, "range constant declaration is incomplete"))?;
                (
                    parent,
                    member,
                    self.associated_constant_with_self(node, member, ty, Some((self_type, base)))?,
                )
            }
            SourceRangeMember::Predecessor {
                parent,
                member,
                signature,
            } => {
                let Some(Kind::Function { result, .. }) = self.facts.types.kind(signature) else {
                    return Err(self.error(node, "range predecessor signature changed"));
                };
                (
                    parent,
                    member,
                    self.pattern_source_function(node, member, result, base)?,
                )
            }
        };
        if self.facts.associated.trait_owner(owner) != Some(parent) {
            return Err(self.error(node, "pattern helper no longer belongs to its actual trait"));
        }
        self.pattern_api_use(node, owner, &site)?;
        let fact = self.facts.node_mut(key).ok_or_else(|| {
            super::Declarations::error(self.report, key, "pattern endpoint facts missing")
        })?;
        fact.reference = Some(Target::Definition(owner));
        Ok(term)
    }
    pub(super) fn bind_pattern_self(
        &mut self,
        node: NodeId,
        rigid: TypeId,
        base: TypeId,
        bindings: &[(ParameterId, Argument)],
    ) -> Result<(), Error> {
        let target = self
            .state
            .infer
            .instantiate(&mut self.facts.types, rigid, bindings)
            .map_err(|e| self.error(node, e))?;
        let base = self.known(node, base)?;
        self.equate(node, target, base)
    }
    fn pattern_api_use(
        &mut self,
        node: NodeId,
        member: DefId,
        site: &crate::resolve::expansion::PatTyRangeSite<'_>,
    ) -> Result<(), Error> {
        let Some(library) = self.report.package.library_policy() else {
            return Ok(());
        };
        let Some(inventory) = library.inventory() else {
            return Ok(());
        };
        // Const policy remains the ordinary call's separate responsibility.
        if let Some(policy) = inventory.declaration(member)
            && matches!(
                policy.level,
                crate::resolve::stability::Level::Unstable { .. }
            )
        {
            let feature = policy.feature_name().map_err(|e| self.error(node, e))?;
            if !site.allows(self.report, self.key(node), feature.spelling())
                && !library
                    .requested_feature(&feature, &mut self.state.steps)
                    .map_err(|e| self.error(node, e))?
            {
                return Err(self.error(
                    node,
                    "generated range helper API requires its exact desugaring allowance or feature",
                ));
            }
        }
        Ok(())
    }
}
