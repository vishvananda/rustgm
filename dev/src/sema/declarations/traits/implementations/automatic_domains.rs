//! Automatic implementation domains preserve ordinary owner and scoped Sized proof.
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    resolve::model::DefKind,
    sema::associated::OwnerKind,
    support::bindings::LangItem,
    types::{
        GenericArgs, ItemId, Kind,
        parameters::{
            ParameterKind,
            predicates::{Bound, Predicate},
        },
        traits::Application,
    },
};

impl Checker<'_> {
    pub(super) fn validate_automatic_domain(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
    ) -> Result<(), Error> {
        let Some((application, _)) = predicate.bound.trait_parts() else {
            return Ok(());
        };
        let trait_id = self
            .facts
            .types
            .trait_application(application)
            .ok_or_else(|| {
                self.error(
                    source,
                    "automatic implementation lost its actual trait application",
                )
            })?
            .item;
        let Some(declaration) = self
            .facts
            .associated
            .owner(DefId(trait_id.0))
            .and_then(|id| self.facts.associated.get(id))
        else {
            return Ok(());
        };
        if !matches!(declaration.kind, OwnerKind::Trait { auto_: true, .. }) {
            return Ok(());
        }
        let local = declaration.source.is_some();
        let kind = self
            .facts
            .types
            .kind(predicate.ty)
            .ok_or_else(|| self.error(source, "automatic implementation subject is absent"))?;
        if !local {
            let (Kind::Adt(item, _) | Kind::Foreign(item)) = kind else {
                return Err(self.error(source, "foreign automatic trait requires an actual local nominal implementation target"));
            };
            let actual = self
                .report
                .resolution
                .definitions
                .get(item.0 as usize)
                .ok_or_else(|| {
                    self.error(source, "automatic nominal target identity is missing")
                })?;
            if actual.node.is_none()
                || !matches!(
                    actual.kind,
                    DefKind::Struct | DefKind::Enum | DefKind::ForeignType
                )
            {
                return Err(self.error(
                    source,
                    "foreign automatic trait implementation target is not local",
                ));
            }
        }
        match kind {
            Kind::Dynamic { .. } => Err(self.error(
                source,
                "automatic traits cannot be implemented for dynamic objects",
            )),
            Kind::Parameter(id) => {
                if matches!(
                    self.facts.types.parameter(id).map(|p| p.kind),
                    Some(ParameterKind::Type { sized: true })
                ) {
                    return Ok(());
                }
                self.automatic_sized_subject(owner, source, predicate.ty)
            }
            Kind::Projection(_) | Kind::Opaque(..) => Err(self.error(
                source,
                "automatic implementation target requires its concrete declared subject",
            )),
            _ => Ok(()),
        }
    }

    fn automatic_sized_subject(
        &mut self,
        owner: DefId,
        source: NodeKey,
        ty: crate::types::TypeId,
    ) -> Result<(), Error> {
        if !self.facts.generics.environments.contains_key(&owner) {
            self.complete_source_environment(source, owner)?;
        }
        let bound = if let Some(catalog) = self.facts.source_bindings.as_deref() {
            let declaration = catalog
                .header_role(LangItem::Sized)
                .map_err(|e| self.error(source, e))?
                .ok_or_else(|| {
                    self.error(
                        source,
                        "automatic generic target requires the actual Sized declaration",
                    )
                })?;
            let payments = crate::sema::header_attributes::source_payments(
                self.facts.must_use_contracts.as_deref(),
            )?;
            Bound::Trait(
                self.facts
                    .types
                    .intern_trait_application_observed::<true>(
                        Application {
                            item: ItemId(declaration.definition.0),
                            arguments: GenericArgs::EMPTY,
                        },
                        &|w, b| payments.charge(w, b),
                    )
                    .map_err(|e| self.error(source, e))?,
            )
        } else {
            Bound::Sized
        };
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = self.declaration_proof(owner, environment);
        let result = proof.prove(&mut self.facts, Predicate { ty, bound });
        self.facts.body_work.canonical_steps = self
            .facts
            .body_work
            .canonical_steps
            .saturating_add(proof.trait_selection_steps());
        self.retain_trait_proof(owner, source, &mut proof);
        if result.map_err(|e| self.error(source, e))? {
            Ok(())
        } else {
            Err(self.error(
                source,
                "automatic implementation generic subject must be Sized",
            ))
        }
    }
}
