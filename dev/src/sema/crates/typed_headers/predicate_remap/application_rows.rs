//! Each complete application is remapped once; clauses retain their own effect.
use super::{Context, Error, Headers, error, s};
use crate::{
    resolve::model::DefKind,
    types::{
        ItemId, Store,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};

impl Context<'_> {
    pub(in crate::sema::crates::typed_headers) fn application(
        &mut self,
        reference: s::TraitApplicationRef,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<Predicate, Error> {
        budget.charge(1)?;
        let index = reference.0 as usize;
        let cached = self
            .applications
            .get(index)
            .ok_or_else(|| error("source clause has an unknown trait application"))?;
        if let Some(predicate) = cached {
            return Ok(*predicate);
        }
        let rows = self.descriptor.rows();
        let application = &rows.trait_applications[index];
        if application.declaration.artifact != rows.policy.provider {
            return Err(error(
                "source clause trait belongs to an unsupported dependency",
            ));
        }
        let definition = self
            .imported
            .generic_definition(
                self.descriptor.identity(),
                application.declaration.declaration,
            )
            .map_err(error)?;
        let declaration = self.catalog.declaration(definition).map_err(error)?;
        self.catalog
            .validate_declaration(declaration, definition, DefKind::Trait)
            .map_err(error)?;
        let arguments = application
            .arguments
            .slice(&rows.arguments)
            .map_err(error)?;
        budget.charge(arguments.len().saturating_add(1))?;
        budget.reserve(arguments.len().saturating_mul(4 * size_of::<Argument>()))?;
        let arguments = arguments
            .iter()
            .map(|argument| match *argument {
                s::Argument::Type(ty) => self.ty(ty).map(Argument::Type),
                s::Argument::Region(region) => self.region(region).map(Argument::Lifetime),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let arguments = store.intern_arguments(&arguments).map_err(error)?;
        let id = store
            .intern_trait_application(Application {
                item: ItemId(definition.0),
                arguments,
            })
            .map_err(error)?;
        let predicate = Predicate {
            ty: self.ty(application.subject)?,
            bound: Bound::Trait(id),
        };
        self.applications[index] = Some(predicate);
        Ok(predicate)
    }
}
