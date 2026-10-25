//! Custom parenthesized traits retain their own direct Output identity and application.
use super::{Checker, DefId, Error, NodeKey};
use crate::{resolve::model::DefKind, types::arguments::Argument};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn require_parenthesized_trait(
        &self,
        key: NodeKey,
        definition: DefId,
    ) -> Result<(), Error> {
        if self.declared_callable(definition) {
            return Ok(());
        }
        if self
            .report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .is_none_or(|definition| definition.kind != DefKind::Trait)
        {
            return Err(self.error(key, "parenthesized arguments require an actual trait"));
        }
        let site = crate::sema::parentheses::name_site(self.report, key)?;
        if crate::sema::parentheses::feature(
            self.report,
            &mut self.report.macro_allowances(),
            site,
            true,
        )? || self
            .facts
            .parenthesis_marker(self.report, definition)?
            .is_some()
        {
            return Ok(());
        }
        Err(self.error(
            key,
            "parenthesized user traits require unboxed_closures or a checked trait marker",
        ))
    }

    pub(in crate::sema::declarations) fn validate_parenthesis_inputs(
        &self,
        key: NodeKey,
        definition: DefId,
    ) -> Result<(), Error> {
        if self.declared_callable(definition) {
            return Ok(());
        }
        let parameters = self
            .facts
            .generics
            .parameters
            .get(&definition)
            .ok_or_else(|| {
                self.error(
                    key,
                    "parenthesized trait requires prepared parameter declarations",
                )
            })?;
        if !matches!(parameters.as_slice(), [parameter] if matches!(parameter.argument, Argument::Type(_)))
        {
            return Err(self.error(
                key,
                "parenthesized user trait input profile requires one type parameter",
            ));
        }
        Ok(())
    }

    pub(super) fn direct_parenthesis_output(
        &mut self,
        key: NodeKey,
        definition: DefId,
    ) -> Result<DefId, Error> {
        self.require_parenthesized_trait(key, definition)?;
        self.validate_parenthesis_inputs(key, definition)?;
        if let Some(member) = self
            .facts
            .must_use_contracts
            .as_deref()
            .and_then(|table| table.parentheses.as_deref())
            .and_then(|table| table.outputs.get(&definition))
            .copied()
        {
            if self.facts.associated.trait_owner(member) != Some(definition) {
                return Err(self.error(key, "cached parenthesized trait Output owner changed"));
            }
            return Ok(member);
        }
        let owner = self
            .facts
            .associated
            .owner(definition)
            .ok_or_else(|| self.error(key, "parenthesized trait has no associated owner"))?;
        let members =
            self.facts.associated.owner_members(owner).ok_or_else(|| {
                self.error(key, "parenthesized trait has no direct member inventory")
            })?;
        let member = members
            .iter()
            .copied()
            .find(|member| {
                let actual = &self.report.resolution.definitions[member.0 as usize];
                actual.kind == DefKind::TypeAlias
                    && self.report.package.names.get(actual.name) == "Output"
            })
            .ok_or_else(|| {
                self.error(
                    key,
                    "parenthesized trait requires its own direct Output associated type",
                )
            })?;
        let source = self.report.resolution.definitions[member.0 as usize]
            .node
            .ok_or_else(|| {
                self.error(
                    key,
                    "imported parenthesized trait Output contract transport is not implemented",
                )
            })?;
        let item = self.item(source)?;
        if item.generics.start != item.generics.end {
            return Err(self.error(
                key,
                "generic parenthesized trait Output profile is not implemented",
            ));
        }
        let outputs = &mut self
            .facts
            .must_use_contracts
            .get_or_insert_default()
            .parentheses
            .get_or_insert_default()
            .outputs;
        if outputs.len() >= 1_000_000 {
            return Err(self.error(key, "parenthesized trait Output storage budget exceeded"));
        }
        outputs.insert(definition, member);
        Ok(member)
    }
}
