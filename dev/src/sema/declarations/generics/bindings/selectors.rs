//! Family equality arguments use the ordinary typed argument worklist and binder.
use super::{Checker, DefId, Error, NodeKey, Type, TypeId, Work, projection, regions};

impl Checker<'_> {
    pub(super) fn prepare_binding_arguments(
        &mut self,
        source: NodeKey,
        application: projection::Application,
        context: regions::Context,
    ) -> Result<bool, Error> {
        let member = DefId(application.item.0);
        if !self.facts.generics.parameters.contains_key(&member) {
            if self.report.resolution.associated_bindings[&source]
                .arguments
                .is_some()
            {
                return Err(self.error(
                    source,
                    "associated binding member has no generic parameters",
                ));
            }
            return Ok(false);
        }
        if self.associated_bindings.prepared_arguments.insert(source) {
            self.work.push(Work::PathArguments(source, member, context));
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn applied_binding(
        &mut self,
        source: NodeKey,
        mut application: projection::Application,
        context: regions::Context,
        original: bool,
    ) -> Result<TypeId, Error> {
        let member = DefId(application.item.0);
        if self.facts.generics.parameters.contains_key(&member) {
            application.arguments = self.application_arguments(source, member, context)?;
            if original {
                application.arguments = self
                    .facts
                    .unreduced
                    .arguments(source)
                    .unwrap_or(application.arguments);
            }
        }
        self.intern(source, Type::Projection(application))
    }
}
