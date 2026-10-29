//! Only completed source-derived vocabulary can consume its selected header grant.
use super::{Body, Builtin, Error, NodeKey, Transaction};
use crate::{
    expand::stream::{Location, OriginKind},
    resolve::{
        bindings::{FeatureName, panic_contracts},
        expansion::Report,
    },
};
impl Report {
    /// Data about an authenticated generated member, never trait/const permission.
    pub(crate) fn derived_const_parameter_impl(&self, site: NodeKey) -> Result<bool, Error> {
        let Some(source) = self.package.source_core() else {
            return Ok(false);
        };
        let error = |message| Error::at(site.file, 0, message);
        source.macros.charge(4, 0).map_err(error)?;
        let unit = self
            .package
            .units
            .get(site.file.0 as usize)
            .ok_or_else(|| error("derived impl lost its actual unit"))?;
        let node = unit
            .configured
            .ast
            .nodes
            .get(site.node.get() as usize)
            .ok_or_else(|| error("derived impl lost its actual item"))?;
        if (node.span.start as usize) < unit.source.original().len() {
            return Ok(false);
        }
        source
            .macros
            .charge(
                8 + unit.origins.len().checked_ilog2().unwrap_or(0) as usize,
                0,
            )
            .map_err(error)?;
        let Some(origin) = self.package.origin(site.file, node.span.start) else {
            return Ok(false);
        };
        if !matches!(origin.kind, OriginKind::Synthetic(_)) {
            return Ok(false);
        }
        let Some(index) = origin.introduction.0.checked_sub(1).map(|n| n as usize) else {
            return Ok(false);
        };
        let member = self
            .invocations
            .get(index)
            .ok_or_else(|| error("derived impl lost its actual invocation"))?;
        let definition = self
            .definitions
            .get(member.definition)
            .ok_or_else(|| error("derived impl lost its actual definition"))?;
        if !matches!(definition.body, Body::Builtin(Builtin::ConstParamTy)) {
            return Ok(false);
        }
        let entry = self.literal_transaction(index)?;
        self.expanded_derive(entry.owner, entry.effect)?;
        if node.span.start < member.generated.start || node.span.end > member.generated.end {
            return Err(error(
                "ConstParamTy impl escaped its actual generated member",
            ));
        }
        Ok(true)
    }
    pub(in crate::resolve::expansion) fn derived_literal_allowance(
        &self,
        site: Location,
        feature: &FeatureName<'_>,
    ) -> Result<Option<bool>, Error> {
        let Some(source) = self.package.source_core() else {
            return Ok(None);
        };
        let error = |message| Error::at(site.file, site.span.start, message);
        let unit = self
            .package
            .units
            .get(site.file.0 as usize)
            .ok_or_else(|| error("derive grant lost its source unit"))?;
        source
            .macros
            .charge(
                1 + unit.origins.len().checked_ilog2().unwrap_or(0) as usize,
                0,
            )
            .map_err(error)?;
        let Some(origin) = self.package.origin(site.file, site.span.start) else {
            return Ok(None);
        };
        if !matches!(origin.kind, OriginKind::Synthetic(_)) {
            return Ok(None);
        }
        let Some(member_index) = origin.introduction.0.checked_sub(1).map(|n| n as usize) else {
            return Ok(None);
        };
        let member = self
            .invocations
            .get(member_index)
            .ok_or_else(|| error("derive grant lost its literal introduction"))?;
        let definition = self
            .definitions
            .get(member.definition)
            .ok_or_else(|| error("derive grant lost its selected definition"))?;
        let Body::Builtin(role) = definition.body else {
            return Ok(None);
        };
        if !crate::expand::builtin::derive_output::MEMBERS.contains(&role) {
            return Ok(None);
        }
        if site.span.start >= site.span.end
            || site.span.start < origin.generated.start
            || site.span.end > origin.generated.end
            || origin.generated.start < member.generated.start
            || origin.generated.end > member.generated.end
            || origin.context.is_none_or(|context| {
                !self
                    .package
                    .marks
                    .is_opaque(context, definition.context, member.context)
            })
        {
            return Err(error(
                "derive literal grant changed its actual generated interval or context",
            ));
        }
        let entry = self.literal_transaction(member_index)?;
        self.expanded_derive(entry.owner, entry.effect)?;
        let node = definition
            .node
            .ok_or_else(|| error("derive grant lost its source declaration"))?;
        let header = panic_contracts::published(
            &self.package,
            node,
            definition.header.as_ref(),
            feature.spelling().len() + 1,
        )?;
        if header.builtin != Some(role) {
            return Err(error("derive literal grant changed its actual header role"));
        }
        Ok(Some(
            definition
                .header
                .as_ref()
                .is_some_and(|header| header.contains(feature)),
        ))
    }
    fn literal_transaction(&self, member_index: usize) -> Result<&Transaction, Error> {
        let member = &self.invocations[member_index];
        let error = |message| Error::at(member.node.file, member.original.start, message);
        let outer_index = member
            .parent
            .0
            .checked_sub(1)
            .ok_or_else(|| error("derive member lost its actual parent"))?
            as usize;
        let outer = self
            .invocations
            .get(outer_index)
            .ok_or_else(|| error("derive member lost its parent invocation"))?;
        if outer.node != member.node
            || outer.original != member.original
            || outer.generated != member.generated
        {
            return Err(error("derive member changed its actual outer invocation"));
        }
        let unit = self.package.unit(member.node.file);
        let owner = unit.owner_indices[member.node.node.get() as usize]
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
            .ok_or_else(|| error("derive member lost its actual owner"))?;
        let transactions = self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or_else(|| error("derive member has no completed transaction"))?;
        let source = self
            .package
            .source_core()
            .ok_or_else(|| error("derive member lost its actual provider"))?;
        for effect in owner.attributes.clone() {
            source.macros.charge(1, 0).map_err(error)?;
            let Some(&position) = transactions.index.get(&(member.node.file, effect)) else {
                continue;
            };
            let entry = &transactions.entries[position];
            if entry.invocation == outer_index
                && entry.owner == member.node
                && entry.contexts.contains(&member_index)
            {
                return Ok(entry);
            }
        }
        Err(error(
            "derive literal has no exact completed member receipt",
        ))
    }
}
