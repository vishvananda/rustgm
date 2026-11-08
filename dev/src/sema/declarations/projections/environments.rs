//! Projection selection depends on a completed immutable caller environment.
#[cfg(test)]
#[path = "../../../../tests/invariants/normalized_impl_headers.rs"]
mod tests;
use super::{Checker, DefId, DefKind, Error, NodeKey, Work, regions};
use crate::sema::declarations::State;
use crate::types::{
    parameters::predicates::{Bound, Predicate},
    substitute::Request,
};

impl Checker<'_> {
    pub(super) fn demand_projection_environment(
        &mut self,
        key: NodeKey,
        request: Request,
    ) -> Result<(), Error> {
        let Some(application) = request.alias.associated() else {
            return Ok(());
        };
        self.demand_projection_subject(key, application.subject)?;
        let predicate = Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        };
        if !predicate
            .contains_parameters(&self.facts.types)
            .map_err(|e| self.error(key, e))?
            && !predicate
                .has_free_regions(&self.facts.types)
                .map_err(|e| self.error(key, e))?
        {
            return Ok(());
        }
        let context = self
            .projections
            .roots
            .get(&key)
            .ok_or_else(|| self.error(key, "missing projection scope"))?
            .context;
        let mut binder = context.binder;
        while let Some(current) = binder {
            if let Some(owner) = self.regions.owner(current) {
                if !self.facts.generics.environments.contains_key(&owner)
                    && !self.projections.raw_environments.contains_key(&owner)
                {
                    match self.report.resolution.definitions[owner.0 as usize].kind {
                        kind @ (DefKind::Function | DefKind::TypeAlias)
                            if kind == DefKind::Function
                                || self.facts.associated.owner(owner).is_some() =>
                        {
                            self.work.push(Work::ProjectionEnvironment(
                                owner,
                                regions::Context {
                                    binder: Some(current),
                                    position: regions::Position::Output,
                                },
                                false,
                            ));
                        }
                        DefKind::Struct | DefKind::Enum => self
                            .work
                            .push(Work::NominalContract(crate::types::ItemId(owner.0), false)),
                        // An impl header naming a projection of its own
                        // parameter, `impl<R: Try> W<R::Output>`, normalizes
                        // it under the impl's bounds as a signature does.
                        DefKind::SelfType if self.implementation(owner) => {
                            self.work.push(Work::ProjectionEnvironment(
                                owner,
                                regions::Context {
                                    binder: Some(current),
                                    position: regions::Position::Impl,
                                },
                                false,
                            ));
                        }
                        _ => {}
                    }
                }
                break;
            }
            binder = self.regions.parent(current);
        }
        Ok(())
    }

    /// Rustc's `Self` in an impl's bounds is the unnormalized header, so an
    /// impl whose bounds name its own `Self` has no environment while that
    /// header forms. Its header projections stay as written, and each is
    /// normalized again once the impl's environment is complete.
    pub(super) fn self_bounded_header(&mut self, context: regions::Context) -> bool {
        let mut binder = context.binder;
        let owner = loop {
            let Some(current) = binder else {
                return false;
            };
            if let Some(owner) = self.regions.owner(current) {
                break owner;
            }
            binder = self.regions.parent(current);
        };
        if self.report.resolution.definitions[owner.0 as usize].kind != DefKind::SelfType
            || self.facts.associated.schema(owner).is_some()
            || !self.implementation(owner)
        {
            return false;
        }
        if let Some(&named) = self.projections.self_bounded.get(&owner) {
            return named;
        }
        let named = self.bounds_name(owner, owner);
        self.projections.self_bounded.insert(owner, named);
        named
    }

    /// A lexical `Self` owns an impl's binder; a trait's owns its own.
    fn implementation(&self, owner: DefId) -> bool {
        self.report.resolution.definitions[owner.0 as usize]
            .node
            .and_then(|node| self.item(node).ok())
            .is_some_and(|item| matches!(item.kind, crate::syntax::item::ItemKind::Impl { .. }))
    }

    fn demand_projection_subject(
        &mut self,
        key: NodeKey,
        subject: crate::types::TypeId,
    ) -> Result<(), Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(());
        }
        let Some(crate::types::Kind::Adt(item, _)) = self.facts.types.kind(subject) else {
            return Ok(());
        };
        if self
            .facts
            .generics
            .nominal_contracts
            .contains(&DefId(item.0))
        {
            return Ok(());
        }
        if !self.nominal_registered(item) {
            return Err(self.error(
                key,
                "projected nominal subject lacks its completed imported or local contract producer",
            ));
        }
        self.work.push(Work::NominalContract(item, false));
        Ok(())
    }

    pub(in super::super) fn complete_projection_environment(
        &mut self,
        owner: DefId,
        context: regions::Context,
        finish: bool,
    ) -> Result<(), Error> {
        let key = self.report.resolution.definitions[owner.0 as usize]
            .node
            .ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing environment owner",
                )
            })?;
        let state = self.projections.environments.get(&owner).copied();
        if state == Some(State::Complete) || self.facts.generics.environments.contains_key(&owner) {
            return Ok(());
        }
        if !finish {
            if state == Some(State::Active) {
                return Err(self.error(
                    key,
                    "projection normalization depends on unfinished caller bounds",
                ));
            }
            self.projections.environments.insert(owner, State::Active);
            self.work
                .push(Work::ProjectionEnvironment(owner, context, true));
            self.schedule_bounds(owner, context)?;
            return Ok(());
        }
        if self.prepare_bound_environment(
            owner,
            context,
            Work::ProjectionEnvironment(owner, context, true),
        )? {
            return Ok(());
        }
        self.complete_bound_environment(owner, context)?;
        self.complete_source_environment(key, owner)?;
        self.projections.environments.insert(owner, State::Complete);
        Ok(())
    }

    pub(in super::super) fn finish_function_environment(
        &mut self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(), Error> {
        if self.projections.environments.get(&owner) == Some(&State::Complete) {
            return Ok(());
        }
        self.complete_source_environment(key, owner)
    }
}
