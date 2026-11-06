//! Projection selection depends on a completed immutable caller environment.
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
                        _ => {}
                    }
                }
                break;
            }
            binder = self.regions.parent(current);
        }
        Ok(())
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
