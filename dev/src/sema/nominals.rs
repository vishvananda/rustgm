//! Nominal use requirements are instantiated once; proofs belong to the caller's environment.
mod requirements;
pub(crate) mod traits;
use super::model::Declarations;
use crate::types::{
    TypeId,
    parameters::predicates::{Bound, Predicate},
};
pub(super) use requirements::{Requirements, requirements};

/// Predicate proofs are reusable only within one immutable declaration environment.
pub(super) struct Proof {
    environment: Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
    shapes: Option<crate::types::parameters::shape::Shapes>,
    copies: crate::types::copy::Copies,
    checked: Requirements,
    traits: Option<Box<traits::State>>,
    scope: Option<(u32, u32)>,
    provisional: bool,
}
impl Proof {
    pub(super) fn for_coherence(
        environment: std::sync::Arc<crate::types::parameters::environment::Environment>,
    ) -> Self {
        Self {
            environment: Some(std::sync::Arc::clone(&environment)),
            shapes: Some(crate::types::parameters::shape::Shapes::new(
                std::sync::Arc::clone(&environment),
            )),
            copies: crate::types::copy::Copies::for_coherence(environment),
            checked: Requirements::default(),
            traits: None,
            scope: None,
            provisional: false,
        }
    }
    pub fn new(
        environment: Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
    ) -> Self {
        let shapes = environment
            .as_ref()
            .map(|env| crate::types::parameters::shape::Shapes::new(std::sync::Arc::clone(env)));
        let copies =
            environment
                .as_ref()
                .map_or_else(crate::types::copy::Copies::default, |environment| {
                    crate::types::copy::Copies::in_environment(std::sync::Arc::clone(environment))
                });
        Self {
            environment,
            shapes,
            copies,
            checked: Requirements::default(),
            traits: None,
            scope: None,
            provisional: false,
        }
    }
    /// Declaration selection may use derive contracts before fields finish.
    /// The mandatory nominal gates still validate every selected derive later.
    pub(super) fn for_unvalidated_implementations(
        environment: Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
    ) -> Self {
        let mut proof = Self::new(environment.clone());
        proof.copies = crate::types::copy::Copies::for_derive(environment);
        proof.provisional = true;
        proof
    }
    pub fn for_declaration(
        environment: Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
        universals: u32,
        first_region: u32,
    ) -> Self {
        let mut proof = Self::new(environment);
        proof.scope = Some((universals, first_region));
        proof
    }
    /// Closed declaration bounds must not justify themselves through their environment.
    pub fn prove_closed_trait(
        &mut self,
        facts: &mut Declarations,
        requirement: Predicate,
    ) -> Result<bool, &'static str> {
        self.trait_state(facts)?.prove(facts, requirement, true)
    }
    pub fn prove_closed_equality(
        &mut self,
        facts: &mut Declarations,
        requirement: Predicate,
    ) -> Result<bool, &'static str> {
        self.trait_state(facts)?
            .prove_equality(facts, requirement, true)
    }
    fn shape(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
    ) -> Result<crate::types::parameters::shape::Properties, &'static str> {
        if let Some(shapes) = &mut self.shapes {
            return shapes.demand(&facts.types, ty);
        }
        Ok(crate::types::parameters::shape::Properties {
            sized: facts
                .types
                .is_sized(ty)
                .ok_or("invalid declaration shape type")?,
            well_formed: facts
                .types
                .is_well_formed(ty)
                .ok_or("invalid declaration shape type")?,
        })
    }
    pub fn check(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<crate::types::parameters::shape::Properties, &'static str> {
        if facts.generics.has_nominal_requirements {
            for requirement in requirements(facts, ty, &mut self.checked)? {
                let valid = self.prove(facts, requirement)?;
                if !valid {
                    return Err(match requirement.bound {
                        Bound::Copy => "nominal type argument must implement Copy",
                        Bound::Clone => "nominal type argument must implement Clone",
                        Bound::Sized => "nominal type argument must be sized",
                        Bound::Trait(_) => {
                            "nominal type argument must implement its declared trait"
                        }
                        Bound::Outlives(_) => "nominal type argument must outlive its bound",
                        Bound::ProjectionEqual(_) => {
                            "associated type must equal its required result"
                        }
                    });
                }
            }
        }
        self.shape(facts, ty)
    }
    pub fn prove(
        &mut self,
        facts: &mut Declarations,
        requirement: Predicate,
    ) -> Result<bool, &'static str> {
        let valid = match requirement.bound {
            Bound::ProjectionEqual(_) => {
                self.trait_state(facts)?
                    .prove_equality(facts, requirement, false)?
            }
            Bound::Trait(_) => self.prove_trait(facts, requirement)?,
            Bound::Sized => self.shape(facts, requirement.ty)?.sized,
            Bound::Copy | Bound::Clone => {
                if !self.provisional {
                    facts.aggregates.prepare(&mut facts.types, requirement.ty)?;
                }
                let abilities = self.copies.demand(
                    &facts.types,
                    &facts.aggregates,
                    &facts.derives,
                    requirement.ty,
                )?;
                if requirement.bound == Bound::Copy {
                    abilities.copy
                } else {
                    abilities.clone_
                }
            }
            Bound::Outlives(_) => {
                return Err("nominal outlives obligation requires its region proof");
            }
        };
        Ok(valid)
    }

    fn prove_trait(
        &mut self,
        facts: &mut Declarations,
        requirement: Predicate,
    ) -> Result<bool, &'static str> {
        let Bound::Trait(application) = requirement.bound else {
            return Err("expected a canonical trait requirement");
        };
        if self
            .environment
            .as_ref()
            .is_some_and(|env| env.assumes_trait(requirement.ty, application))
        {
            return Ok(true);
        }
        self.trait_state(facts)?.prove(facts, requirement, false)
    }

    fn trait_state(
        &mut self,
        facts: &mut Declarations,
    ) -> Result<&mut traits::State, &'static str> {
        if self.traits.is_none() {
            self.traits = Some(Box::new(traits::State::new(
                facts,
                self.environment.clone(),
                self.scope,
            )?));
        }
        self.traits
            .as_deref_mut()
            .ok_or("missing nominal trait proof")
    }

    /// Retain every selected lifetime obligation, including fresh existential identities.
    pub fn take_trait_proof(&mut self) -> Option<super::traits::obligations::Obligations> {
        self.traits.take().map(|state| state.finish())
    }
}
