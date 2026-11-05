//! Async nominal ownership shares Drop coverage without registering a sync call.
use super::{
    super::{Checker, Error},
    coverage::NominalContract,
};
use crate::{
    resolve::{model::Target, unit::NodeKey},
    sema::{
        associated::{ImplPolarity, OwnerKind},
        destructors::{AsyncImplementation, LifetimeContract},
    },
    support::bindings::LangItem,
    types::{Kind, arguments::Argument, outlives::Queries},
};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn prepare_async_destructors(&mut self) -> Result<(), Error> {
        let role = self
            .facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.validated_language(LangItem::AsyncDrop))
            .transpose()
            .map_err(Error::unlocated)?
            .flatten();
        if let Some(role) = role {
            for index in 0..self.facts.associated.owners().len() {
                let Some(owner) = self
                    .facts
                    .associated
                    .source_at(index)
                    .map_err(Error::unlocated)?
                else {
                    continue;
                };
                let OwnerKind::Impl {
                    trait_type: Some(path),
                    polarity,
                    ..
                } = owner.kind
                else {
                    continue;
                };
                let path = NodeKey {
                    node: path.node,
                    ..owner.source
                };
                if self.facts.node(path).and_then(|f| f.reference)
                    != Some(Target::Definition(role.definition))
                {
                    continue;
                }
                if polarity != ImplPolarity::Positive {
                    return Err(
                        self.error(owner.source, "AsyncDrop requires a positive implementation")
                    );
                }
                self.prepare_async_destructor(owner)?;
            }
        }
        if let Some(catalog) = self.facts.source_bindings.as_deref_mut() {
            catalog
                .complete_async_destructor_registration()
                .map_err(Error::unlocated)?;
        }
        Ok(())
    }

    fn prepare_async_destructor(
        &mut self,
        owner: crate::sema::associated::SourceOwner,
    ) -> Result<(), Error> {
        let schema = self
            .facts
            .associated
            .schema(owner.self_definition)
            .ok_or_else(|| {
                self.error(
                    owner.source,
                    "AsyncDrop requires its actual prepared impl schema",
                )
            })?;
        let (target, captures) = (schema.self_type, schema.captures);
        let Some(Kind::Adt(item, arguments)) = self.facts.types.kind(target) else {
            return Err(self.error(
                owner.source,
                "AsyncDrop requires a local struct or enum target",
            ));
        };
        let nominal = self.report.resolution.definitions.get(item.0 as usize);
        if nominal.is_none_or(|d| {
            !matches!(
                d.kind,
                crate::resolve::model::DefKind::Struct | crate::resolve::model::DefKind::Enum
            ) || d.node.is_none()
        }) {
            return Err(self.error(
                owner.source,
                "AsyncDrop requires a local struct or enum target",
            ));
        }
        if self
            .report
            .package
            .requested_async_drop()
            .map_err(|e| self.error(owner.source, e))?
            && self.facts.destructors.item(item).is_none()
        {
            return Err(self.error(
                owner.source,
                "AsyncDrop requires a synchronous Drop implementation",
            ));
        }
        let coverage = self
            .drop_target_coverage(owner.self_definition, item, arguments, captures)
            .map_err(|e| self.error(owner.source, e))?;
        self.facts
            .destructors
            .insert_async(AsyncImplementation {
                self_definition: owner.self_definition,
                item,
                coverage,
            })
            .map_err(|e| self.error(owner.source, e))?;
        Ok(())
    }

    pub(in crate::sema::declarations) fn validate_async_destructors(
        &mut self,
    ) -> Result<(), Error> {
        let owners = self
            .facts
            .destructors
            .async_implementations()
            .map(|i| (i.self_definition, i.item))
            .collect::<Vec<_>>();
        let mut proofs = crate::support::hash::HashMap::default();
        for &(owner, item) in &owners {
            let (source, schema) = self.async_destructor_schema(owner)?;
            if let Some(proof) = self.validate_destructor_traits(NominalContract {
                owner,
                scope: owner,
                item,
                source,
                parameters: schema.captures,
            })? {
                proofs.insert(owner, proof);
            }
        }
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        for (owner, _) in owners {
            let (source, schema) = self.async_destructor_schema(owner)?;
            let Some(Kind::Adt(_, arguments)) = self.facts.types.kind(schema.self_type) else {
                return Err(self.error(source, "AsyncDrop lost its nominal target"));
            };
            let arguments = self
                .facts
                .types
                .arguments(arguments)
                .ok_or_else(|| self.error(source, "AsyncDrop lost its actual target arguments"))?
                .filter_map(|argument| {
                    if let Argument::Lifetime(region) = argument {
                        Some(region)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let assumptions = queries
                .symbolic_requirements(schema.self_type)
                .map_err(|e| self.error(source, e))?;
            crate::sema::destructors::check_bounds(
                LifetimeContract {
                    parameters: schema.captures,
                    bounds: &schema.bounds,
                },
                &arguments,
                &assumptions,
                &self.facts.types,
                self.facts
                    .generics
                    .predicates_in_scope(owner)
                    .map(|p| p.value),
                proofs.get(&owner),
            )
            .map_err(|e| self.error(source, e))?;
        }
        Ok(())
    }
    fn async_destructor_schema(
        &self,
        owner: crate::resolve::model::DefId,
    ) -> Result<(NodeKey, &crate::sema::associated::Schema), Error> {
        let source = self
            .facts
            .associated
            .owner(owner)
            .and_then(|id| self.facts.associated.get(id))
            .and_then(|owner| owner.source)
            .ok_or_else(|| Error::unlocated("AsyncDrop lost its actual source owner"))?;
        let schema = self
            .facts
            .associated
            .schema(owner)
            .ok_or_else(|| self.error(source, "AsyncDrop lost its actual impl schema"))?;
        Ok((source, schema))
    }
}
