//! Retirement authenticates the selected Attribute declaration without an execution frame.
use super::{
    Error, Removal, Report,
    demand::{self, Work},
};
use crate::{
    expand::{
        crate_syntax::AttributeEffect,
        marks::{ExpansionId, SyntaxContextId},
        textual::Scope,
    },
    resolve::{
        bindings::panic_contracts::HeaderData,
        model,
        package::ModuleId,
        unit::{NodeKey, Unit},
    },
    support::{edition::Edition, names::NameId},
    syntax::token_tree::Tree,
};
use std::{cell::Cell, rc::Rc};
pub(super) struct Identity {
    node: Option<NodeKey>,
    module: ModuleId,
    name: NameId,
    context: SyntaxContextId,
    introduction: ExpansionId,
    preceding: Scope,
    local_inner: bool,
    edition: Edition,
    edition_panic: bool,
    header: Option<Rc<HeaderData>>,
    scope: ScopeIdentity,
    capture: u32,
    attribute: crate::expand::configured::ConfiguredAttribute,
    trees: Vec<Tree>,
    forest_end: u32,
    origins: Vec<Vec<super::saved_uses::OriginWitness>>,
}
#[derive(PartialEq, Eq)]
struct ScopeIdentity {
    parent: Option<model::ScopeId>,
    module: ModuleId,
    barrier: bool,
    associated: bool,
    trait_access: Option<model::Access>,
    nominal_self: Option<model::DefId>,
    label_barrier: bool,
}
impl From<&model::Scope> for ScopeIdentity {
    fn from(scope: &model::Scope) -> Self {
        Self {
            parent: scope.parent,
            module: scope.module,
            barrier: scope.barrier,
            associated: scope.associated,
            trait_access: scope.trait_access,
            nominal_self: scope.nominal_self,
            label_barrier: scope.label_barrier,
        }
    }
}
impl Identity {
    #[cfg(test)]
    pub(super) fn retained_payload(&self) -> usize {
        crate::support::retained::vector(&self.trees)
            + crate::support::retained::vector(&self.origins)
            + self
                .origins
                .iter()
                .map(crate::support::retained::vector)
                .sum::<usize>()
    }
    pub(super) fn capture<'unit>(
        unit: &'unit Unit,
        removal: &Removal,
        definition: &super::super::Definition,
        scope: &model::Scope,
        work: &Cell<Work>,
        other: impl Fn(crate::resolve::unit::FileId) -> Option<&'unit Unit>,
    ) -> Result<Self, Error> {
        let effect = &unit.configured.effects[removal.effect];
        let forest = unit
            .configured
            .ast
            .attributes
            .get(effect.capture as usize)
            .ok_or_else(|| {
                Error::at(
                    removal.owner.file,
                    0,
                    "Test selected effect lost its capture",
                )
            })?;
        demand::charge(
            work,
            removal.owner.file,
            1 + forest.forest.nodes().len(),
            forest.forest.nodes().len() * size_of::<Tree>(),
        )?;
        let mut origins = Vec::new();
        for tree in forest.forest.nodes() {
            demand::charge(
                work,
                removal.owner.file,
                1,
                4 * size_of::<Vec<super::saved_uses::OriginWitness>>(),
            )?;
            origins.push(super::saved_uses::origin_chain(
                unit,
                removal.owner.file,
                tree.span,
                work,
                &other,
            )?);
        }
        Ok(Self {
            node: definition.node,
            module: definition.module,
            name: definition.name,
            context: definition.context,
            introduction: definition.introduction,
            preceding: definition.preceding_scope,
            local_inner: definition.local_inner_macros,
            edition: definition.edition,
            edition_panic: definition.edition_panic,
            header: definition.header.clone(),
            scope: ScopeIdentity::from(scope),
            capture: effect.capture,
            attribute: effect.attribute,
            trees: forest.forest.nodes().to_vec(),
            forest_end: forest.forest.source_end(),
            origins,
        })
    }
    pub(super) fn validate(
        &self,
        report: &Report,
        removal: &Removal,
        work: &Cell<Work>,
    ) -> Result<(), Error> {
        demand::charge(work, removal.owner.file, 1 + self.trees.len(), 0)?;
        let definition = &report.definitions[removal.definition];
        let unit = report.package.unit(removal.owner.file);
        let effect = unit.configured.effects.get(removal.effect);
        let same_header = match (&self.header, &definition.header) {
            (Some(left), Some(right)) => Rc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        };
        if definition.node != self.node
            || definition.module != self.module
            || definition.name != self.name
            || definition.context != self.context
            || definition.introduction != self.introduction
            || definition.preceding_scope != self.preceding
            || definition.local_inner_macros != self.local_inner
            || definition.edition != self.edition
            || definition.edition_panic != self.edition_panic
            || !same_header
            || report
                .resolution
                .scopes
                .get(removal.scope.0 as usize)
                .is_none_or(|scope| ScopeIdentity::from(scope) != self.scope)
            || effect.is_none_or(|effect: &AttributeEffect| {
                effect.capture != self.capture || effect.attribute != self.attribute
            })
            || unit
                .configured
                .ast
                .attributes
                .get(self.capture as usize)
                .is_none_or(|capture| capture.forest.nodes() != self.trees.as_slice())
            || unit
                .configured
                .ast
                .attributes
                .get(self.capture as usize)
                .is_none_or(|capture| capture.forest.source_end() != self.forest_end)
        {
            return Err(Error::at(
                removal.owner.file,
                0,
                "removed Test selected declaration/caller/effect changed",
            ));
        }
        for chain in &self.origins {
            for origin in chain {
                origin.validate(report, work)?;
            }
        }
        Ok(())
    }
}
