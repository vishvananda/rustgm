//! Retain complete impl applications; headers alone do not prove an implementation.
mod methods;
mod validation;
use super::super::{Checker, DefId, Error, ItemId, NodeKey, Work};
use crate::{
    resolve::model::DefKind,
    sema::{associated::OwnerKind, traits::implementations::Implementation},
    types::parameters::{
        predicates::{Bound, Predicate},
        uses::Uses,
    },
};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn prepare_trait_implementations(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let mut unsupported = None;
        for index in 0..self.facts.associated.owners().len() {
            let owner = self.facts.associated.owners()[index];
            let OwnerKind::Impl {
                target,
                trait_type: Some(path),
                unsafe_,
                negative,
            } = owner.kind
            else {
                continue;
            };
            if self
                .facts
                .destructors
                .self_type(owner.self_definition)
                .is_some()
            {
                continue;
            }
            if negative {
                return Err(self.error(
                    owner.source,
                    "negative trait implementations require trait coherence",
                ));
            }
            let (path, definition) = self.resolved_trait_path(path)?;
            if self.impl_trait_safety(path, definition)? != unsafe_ {
                return Err(self.error(
                    owner.source,
                    "trait and implementation safety declarations disagree",
                ));
            }
            // Associated constants retain their explicit elaboration boundary.
            let unsupported = unsupported.get_or_insert_with(|| {
                self.facts
                    .associated
                    .members()
                    .iter()
                    .filter(|member| {
                        !matches!(
                            self.report.resolution.definitions[member.definition.0 as usize].kind,
                            DefKind::Function | DefKind::TypeAlias
                        )
                    })
                    .map(|member| member.owner)
                    .collect::<std::collections::HashSet<_>>()
            });
            if [owner.self_definition, definition].into_iter().any(|id| {
                self.facts
                    .associated
                    .owner(id)
                    .is_some_and(|id| unsupported.contains(&id))
            }) {
                return Err(self.error(
                    owner.source,
                    "trait implementation requires conformance and selection semantics",
                ));
            }
            let header =
                self.impl_header(owner.self_definition, owner.source, target, steps, limit)?;
            let ty = header.schema.self_type;
            let context = header.context;
            self.complete_impl_owner(owner.self_definition, owner.source, header, steps, limit)?;
            self.work
                .push(Work::PathArguments(path, definition, context));
            self.drain(steps, limit)?;
            let application = self.trait_path(path, definition, ty, context)?;
            let predicate = Predicate {
                ty,
                bound: Bound::Trait(application),
            };
            self.trait_impl_parameters(owner.self_definition, owner.source, predicate)?;
            self.facts
                .generics
                .implementations
                .get_or_insert_with(Default::default)
                .prepare(
                    &mut self.facts.types,
                    ItemId(definition.0),
                    Implementation {
                        owner: owner.self_definition,
                        source: owner.source,
                        predicate,
                        methods: Vec::new(),
                        types: Vec::new(),
                    },
                )
                .map_err(|e| self.error(owner.source, e))?;
        }
        Ok(())
    }

    fn impl_trait_safety(&self, path: NodeKey, definition: DefId) -> Result<bool, Error> {
        let trait_owner = self
            .facts
            .associated
            .owner(definition)
            .and_then(|id| self.facts.associated.get(id))
            .ok_or_else(|| {
                self.error(
                    path,
                    "trait implementation requires a local trait declaration",
                )
            })?;
        let OwnerKind::Trait { unsafe_, .. } = trait_owner.kind else {
            return Err(self.error(path, "implementation path is not a trait"));
        };
        Ok(unsafe_)
    }

    fn trait_impl_parameters(
        &self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
    ) -> Result<(), Error> {
        let mut uses = Uses::constraining();
        uses.predicate(&self.facts.types, predicate)
            .map_err(|e| self.error(source, e))?;
        for id in self
            .facts
            .generics
            .parameter_ids(&self.facts.types, owner)
            .map_err(|e| self.error(source, e))?
        {
            if !uses.parameters.contains(&id) && !uses.constants.contains(&id) {
                return Err(self.error(
                    source,
                    "impl type and const parameters must constrain Self or the trait application",
                ));
            }
        }
        Ok(())
    }
}
