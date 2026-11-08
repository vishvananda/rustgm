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

/// Where an automatic implementation was declared. An imported one names an
/// auto trait of its own packet, so the rule is that crate's local rule.
#[derive(Clone, Copy)]
enum Site {
    Source(NodeKey),
    Imported,
}

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
        self.automatic_subject(owner, Site::Source(source), predicate.ty, kind)
    }

    /// Version thirteen: an imported impl of its own packet's auto trait,
    /// for which that trait is local. The local rule is re-proved here.
    pub(super) fn validate_imported_automatic_domain(
        &mut self,
        owner: DefId,
        ty: crate::types::TypeId,
    ) -> Result<(), Error> {
        let kind = self
            .facts
            .types
            .kind(ty)
            .ok_or_else(|| Error::unlocated("automatic implementation subject is absent"))?;
        self.automatic_subject(owner, Site::Imported, ty, kind)
    }

    /// The local auto Self rule: never a dynamic object, a projection or an
    /// opaque type, and a generic subject only when it is Sized.
    fn automatic_subject(
        &mut self,
        owner: DefId,
        site: Site,
        ty: crate::types::TypeId,
        kind: Kind,
    ) -> Result<(), Error> {
        match kind {
            Kind::Dynamic { .. } => Err(self.site_error(
                site,
                "automatic traits cannot be implemented for dynamic objects",
            )),
            Kind::Parameter(id) => {
                if matches!(
                    self.facts.types.parameter(id).map(|p| p.kind),
                    Some(ParameterKind::Type { sized: true })
                ) {
                    return Ok(());
                }
                self.automatic_sized_subject(owner, site, ty)
            }
            Kind::Projection(_) | Kind::Opaque(..) => Err(self.site_error(
                site,
                "automatic implementation target requires its concrete declared subject",
            )),
            _ => Ok(()),
        }
    }

    fn site_error(&self, site: Site, message: impl Into<String>) -> Error {
        match site {
            Site::Source(source) => self.error(source, message),
            Site::Imported => Error::unlocated(message),
        }
    }

    fn automatic_sized_subject(
        &mut self,
        owner: DefId,
        site: Site,
        ty: crate::types::TypeId,
    ) -> Result<(), Error> {
        let bound = match site {
            Site::Source(source) => self.automatic_sized_bound(owner, source)?,
            Site::Imported => imported_sized_bound(&self.facts).map_err(Error::unlocated)?,
        };
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = self.declaration_proof(owner, environment);
        let result = proof.prove(&mut self.facts, Predicate { ty, bound });
        self.facts.body_work.canonical_steps = self
            .facts
            .body_work
            .canonical_steps
            .saturating_add(proof.trait_selection_steps());
        match site {
            Site::Source(source) => self.retain_trait_proof(owner, source, &mut proof),
            Site::Imported => {
                if proof
                    .take_trait_proof()
                    .is_some_and(|evidence| !evidence.is_empty())
                {
                    return Err(Error::unlocated(
                        "imported automatic Sized evidence requires its wider profile",
                    ));
                }
            }
        }
        if result.map_err(|e| self.site_error(site, e))? {
            Ok(())
        } else {
            Err(self.site_error(
                site,
                "automatic implementation generic subject must be Sized",
            ))
        }
    }

    fn automatic_sized_bound(&mut self, owner: DefId, source: NodeKey) -> Result<Bound, Error> {
        if !self.facts.generics.environments.contains_key(&owner) {
            self.complete_source_environment(source, owner)?;
        }
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(Bound::Sized);
        };
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
        Ok(Bound::Trait(
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
        ))
    }
}

/// A source client's Sized goal is the imported Sized role's own installed
/// Self bound, as decoded original sizing requires: no new application is
/// interned and so nothing is paid.
fn imported_sized_bound(facts: &crate::sema::model::Declarations) -> Result<Bound, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(Bound::Sized);
    };
    let role = catalog
        .validated_role(LangItem::Sized)?
        .ok_or("automatic generic target requires the actual Sized declaration")?;
    facts.associated.require_imported_role(role)?;
    let bound = facts
        .generics
        .trait_self_bounds
        .get(&role.definition)
        .ok_or("imported automatic sizing lacks its actual Sized schema")?
        .bound;
    let Bound::Trait(id) = bound else {
        return Err("imported automatic sizing changed its actual Sized goal");
    };
    if facts.types.trait_application(id).is_none_or(|application| {
        application.item.0 != role.definition.0
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|arguments| arguments.len() != 0)
    }) {
        return Err("imported automatic sizing belongs to a different trait/application");
    }
    Ok(bound)
}
