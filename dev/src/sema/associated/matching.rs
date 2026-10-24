//! Match implementation identifiers in the actual trait's existing member table.
use super::Inventory;
use crate::resolve::{
    expansion::Report,
    model::{DefId, DefKind, ItemContext, Origin},
};

#[cfg(test)]
#[path = "../../../tests/invariants/adjusted_members.rs"]
mod tests;

impl Inventory {
    /// Prepare identity links before a header becomes available to projection
    /// queries. This does not complete the impl or prove any result contract.
    pub(in crate::sema) fn bind_type_members(
        &mut self,
        report: &Report,
        trait_item: DefId,
        implementation: DefId,
    ) -> Result<(), (DefId, &'static str)> {
        let owner = self.owner(implementation).ok_or((
            implementation,
            "missing associated type implementation owner",
        ))?;
        for index in 0..self.owner_members[owner.0 as usize].len() {
            let actual = self.owner_members[owner.0 as usize][index];
            if report.resolution.definitions[actual.0 as usize].kind != DefKind::TypeAlias {
                continue;
            }
            let expected = self
                .matching_member(report, trait_item, actual)
                .map_err(|error| (actual, error))?
                .ok_or((actual, "implementation type is not a member of the trait"))?;
            if self
                .type_members
                .insert((owner, expected), actual)
                .is_some_and(|previous| previous != actual)
            {
                return Err((
                    actual,
                    "multiple implementation members match the same trait member",
                ));
            }
        }
        Ok(())
    }

    /// This is declaration identity only; signatures, defaults and bodies still
    /// require their ordinary conformance and execution proofs.
    pub(in crate::sema) fn matching_member(
        &self,
        report: &Report,
        trait_item: DefId,
        actual: DefId,
    ) -> Result<Option<DefId>, &'static str> {
        let context = report
            .definition_item_context(actual)
            .map_err(|_| "invalid implementation member context")?;
        self.matching_context(report, trait_item, actual, context)
    }

    fn matching_context(
        &self,
        report: &Report,
        trait_item: DefId,
        actual: DefId,
        context: ItemContext,
    ) -> Result<Option<DefId>, &'static str> {
        let definitions = &report.resolution.definitions;
        let container = definitions
            .get(trait_item.0 as usize)
            .ok_or("unknown implementation trait container")?;
        let declaration = definitions
            .get(actual.0 as usize)
            .ok_or("unknown implementation member")?;
        if !matches!(
            declaration.kind,
            DefKind::Function | DefKind::TypeAlias | DefKind::Const
        ) {
            return Err("unsupported implementation member kind");
        }
        let scope = container
            .members
            .ok_or("implementation trait has no member scope")?;
        let caller = report
            .resolution
            .scopes
            .get(declaration.scope.0 as usize)
            .ok_or("implementation member has no actual scope")?
            .module;
        let adjusted = report.adjust_item(trait_item, context, caller)?;
        let key = (
            scope,
            declaration.name,
            declaration.namespace,
            adjusted.context,
        );
        let Some(binding) = report.resolution.bindings.get_key(key) else {
            return Ok(None);
        };
        let expected = definitions
            .get(binding.target.0 as usize)
            .ok_or("trait member binding has an unknown declaration")?;
        if binding.ambiguous
            || binding.origin != Origin::Direct
            || expected.kind != declaration.kind
            || self.trait_owner(binding.target) != Some(trait_item)
        {
            return Ok(None);
        }
        Ok(Some(binding.target))
    }
}
