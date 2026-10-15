//! All impl kinds share generic/lifetime elaboration before their own conformance checks.
use super::{BinderId, Checker, DefId, Error, NodeKey, regions};
use crate::sema::associated::Schema;

pub(super) struct Header {
    pub binder: BinderId,
    pub context: regions::Context,
    pub schema: Schema,
}

impl Checker<'_> {
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
        self.schedule_bounds(owner, header.context)?;
        self.drain(steps, limit)?;
        self.facts
            .generics
            .complete_environment(&mut self.facts.types, owner)
            .map_err(|e| self.error(source, e))
    }
}
