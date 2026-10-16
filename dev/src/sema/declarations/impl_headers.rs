//! All impl kinds share generic/lifetime elaboration before their own conformance checks.
use super::{BinderId, Checker, DefId, Error, NodeKey, Work, regions};
use crate::sema::associated::Schema;
use crate::types::parameters::{
    constraints::Constraints,
    predicates::{Bound, Predicate},
    uses::Uses,
};

pub(super) struct Header {
    pub binder: BinderId,
    pub context: regions::Context,
    pub schema: Schema,
}

impl Checker<'_> {
    pub(super) fn impl_parameters(
        &self,
        owner: DefId,
        source: NodeKey,
        header: Predicate,
    ) -> Result<(), Error> {
        let error = |e| self.error(source, e);
        let types = &self.facts.types;
        let parameters = self
            .facts
            .generics
            .parameter_ids(types, owner)
            .map_err(error)?;
        let mut uses = Uses::constraining();
        uses.predicate(types, header).map_err(error)?;
        if parameters
            .iter()
            .all(|id| uses.parameters.contains(id) || uses.constants.contains(id))
        {
            return Ok(());
        }
        let mut constraints = Constraints::new(types, header, uses).map_err(error)?;
        for predicate in self.facts.generics.predicates_in_scope(owner) {
            let original = self
                .facts
                .unreduced
                .predicate(predicate.source)
                .unwrap_or(predicate.value);
            constraints.equality(types, original).map_err(error)?;
        }
        constraints.solve().map_err(error)?;
        if parameters.into_iter().all(|id| constraints.determines(id)) {
            return Ok(());
        }
        Err(error(if matches!(header.bound, Bound::Trait(_)) {
            "impl type and const parameters must constrain Self or the trait application"
        } else {
            "impl type and const parameters must constrain its target"
        }))
    }

    pub(super) fn impl_header(
        &mut self,
        owner: DefId,
        source: NodeKey,
        target: NodeKey,
        steps: &mut usize,
        limit: usize,
    ) -> Result<Header, Error> {
        let declared = self.declared_regions(source, self.item(source)?.generics)?;
        let binder = self.regions.binder(None, declared.clone());
        self.regions.associate(binder, owner);
        for parameter in declared {
            self.regions
                .named(Some(binder), parameter)
                .map_err(|e| self.error(source, e))?;
        }
        let bounds = self.function_bounds(source, binder)?;
        let context = regions::Context {
            binder: Some(binder),
            position: regions::Position::Impl,
        };
        self.declaration_parameters(owner, source, context)?;
        self.drain(steps, limit)?;
        self.node_work(target.file, target.node, context);
        self.drain(steps, limit)?;
        Ok(Header {
            binder,
            context,
            schema: Schema {
                self_type: self.ty(target)?,
                captures: self.regions.count(binder),
                bounds,
            },
        })
    }

    pub(super) fn complete_impl_owner(
        &mut self,
        owner: DefId,
        source: NodeKey,
        header: Header,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let target = header.schema.self_type;
        self.facts
            .associated
            .declare(owner, header.schema)
            .map_err(|e| self.error(source, e))?;
        self.owner_binders.insert(owner, header.binder);
        self.complete(owner, source, target)?;
        self.work
            .push(Work::ProjectionEnvironment(owner, header.context, false));
        self.drain(steps, limit)?;
        Ok(())
    }
}
