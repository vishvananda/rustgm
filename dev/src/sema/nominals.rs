//! Nominal use requirements are instantiated once; proofs belong to the caller's environment.
mod formation;
mod requirements;
pub(in crate::sema) mod sizes;
mod terms;
pub(crate) mod traits;
use super::model::Declarations;
use crate::types::{
    TypeId,
    parameters::{
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
};
pub(super) use requirements::{Requirements, requirements};

/// Predicate proofs are reusable only within one immutable declaration environment.
pub(super) struct Proof {
    environment: Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
    shapes: Option<Box<crate::types::parameters::shape::Shapes>>,
    term_shapes: Option<Box<crate::types::parameters::shape::Shapes>>,
    builtin_inputs: Option<Box<terms::Inputs>>,
    copies: crate::types::copy::Copies,
    checked: Requirements,
    traits: Option<Box<traits::State>>,
    scope: Option<(u32, u32)>,
    provisional: bool,
}
impl Proof {
    pub(super) fn resume_body_shapes(&mut self, first: u32) -> Result<(), &'static str> {
        if self.traits.is_some() || self.scope.is_none_or(|(count, _)| count != 0) {
            return Err("body shape proof was not published before resumption");
        }
        self.scope = Some((0, first));
        Ok(())
    }
    pub(super) fn normalize(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<TypeId, &'static str> {
        if facts.types.contains_projections(ty) != Some(true) {
            return Ok(ty);
        }
        self.trait_state(facts)?.normalize(facts, ty)
    }

    pub(super) fn normalize_builtin(
        &mut self,
        facts: &mut Declarations,
        mut requirement: Predicate,
    ) -> Result<Predicate, &'static str> {
        if matches!(
            requirement.bound,
            Bound::Copy | Bound::Clone | Bound::Sized | Bound::Outlives(_)
        ) {
            requirement.ty = self.normalize(facts, requirement.ty)?;
        }
        Ok(requirement)
    }
    pub(super) fn for_coherence(
        environment: std::sync::Arc<crate::types::parameters::environment::Environment>,
    ) -> Self {
        Self {
            environment: Some(std::sync::Arc::clone(&environment)),
            shapes: Some(Box::new(crate::types::parameters::shape::Shapes::new(
                std::sync::Arc::clone(&environment),
            ))),
            term_shapes: None,
            builtin_inputs: None,
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
        let shapes = environment.as_ref().map(|env| {
            Box::new(crate::types::parameters::shape::Shapes::new(
                std::sync::Arc::clone(env),
            ))
        });
        let copies =
            environment
                .as_ref()
                .map_or_else(crate::types::copy::Copies::default, |environment| {
                    crate::types::copy::Copies::in_environment(std::sync::Arc::clone(environment))
                });
        Self {
            environment,
            shapes,
            term_shapes: None,
            builtin_inputs: None,
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
    /// A scratch declaration query starts beyond this proof's retained witnesses.
    pub fn region_frontier(&mut self, facts: &mut Declarations) -> Result<u32, &'static str> {
        if self.scope.is_none() {
            return Err("scoped declaration queries require an explicit proof scope");
        }
        Ok(self.trait_state(facts)?.region_frontier())
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
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<crate::types::parameters::shape::Properties, &'static str> {
        self.shape_inside(facts, ty, None)
    }
    fn shape_inside(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
        count: Option<u32>,
    ) -> Result<crate::types::parameters::shape::Properties, &'static str> {
        if count.is_some() && self.term_shapes.is_none() {
            let environment = if let Some(environment) = &self.environment {
                std::sync::Arc::clone(environment)
            } else {
                std::sync::Arc::new(crate::types::parameters::environment::Environment::new(
                    &facts.types,
                    &[],
                )?)
            };
            self.term_shapes = Some(Box::new(
                crate::types::parameters::shape::Shapes::for_clause_terms(environment),
            ));
        }
        if count.is_none()
            && self.shapes.is_none()
            && facts.types.contains_projected_sizes(ty) == Some(true)
        {
            let environment = std::sync::Arc::new(
                crate::types::parameters::environment::Environment::new(&facts.types, &[])?,
            );
            self.shapes = Some(Box::new(crate::types::parameters::shape::Shapes::new(
                environment,
            )));
        }
        let projected = facts.types.contains_projections(ty) == Some(true);
        if let Some(properties) = self.shape_properties(facts, ty, count, false)?
            && (!projected || properties.sized && properties.well_formed)
        {
            return Ok(properties);
        }
        let ty = if let Some(count) = count {
            self.normalize_clause_term(facts, ty, count)?
        } else {
            self.normalize(facts, ty)?
        };
        if let Some(properties) = self.shape_properties(facts, ty, count, projected)? {
            return Ok(properties);
        }
        if self.shapes.is_some() || count.is_some() {
            return Err("associated type shape checking requires normalization");
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
    fn shape_properties(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
        count: Option<u32>,
        normalized: bool,
    ) -> Result<Option<crate::types::parameters::shape::Properties>, &'static str> {
        use crate::types::parameters::shape::Progress;
        let shapes = if count.is_some() {
            self.term_shapes.take()
        } else {
            self.shapes.take()
        };
        let Some(mut shapes) = shapes else {
            return Ok(None);
        };
        let result = loop {
            match shapes.advance(&facts.types, ty, u16::from(count.is_some()), normalized)? {
                Progress::Ready(properties) => break Some(properties),
                Progress::Projection => break None,
                Progress::Size(request) => {
                    let sized = self.trait_state(facts)?.shape_size(
                        facts,
                        &mut shapes,
                        request,
                        count,
                        normalized,
                    )?;
                    shapes.answer_size(&facts.types, normalized, request, sized)?;
                }
            }
        };
        if count.is_some() {
            self.term_shapes = Some(shapes);
        } else {
            self.shapes = Some(shapes);
        }
        Ok(result)
    }
    pub fn check(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<crate::types::parameters::shape::Properties, &'static str> {
        self.check_inside(facts, ty, None)
    }
    fn check_inside(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
        count: Option<u32>,
    ) -> Result<crate::types::parameters::shape::Properties, &'static str> {
        facts
            .lifetimes
            .nominal_requirements
            .prepare(&mut facts.types, ty)?;
        if facts.generics.has_nominal_requirements {
            for requirement in requirements::requirements_inside(
                facts,
                ty,
                u16::from(count.is_some()),
                &mut self.checked,
            )? {
                let valid = self.prove_clause(facts, requirement)?;
                if !valid {
                    let Clause::Predicate(requirement) = requirement else {
                        return Err("nominal type argument must satisfy its quantified bound");
                    };
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
        self.shape_inside(facts, ty, count)
    }
    pub fn prove_clause(
        &mut self,
        facts: &mut Declarations,
        requirement: Clause,
    ) -> Result<bool, &'static str> {
        match requirement {
            Clause::Predicate(predicate) => self.prove(facts, predicate),
            Clause::ForAll(_) => self
                .trait_state(facts)?
                .prove_clause(facts, requirement, false),
        }
    }
    pub fn prove(
        &mut self,
        facts: &mut Declarations,
        requirement: Predicate,
    ) -> Result<bool, &'static str> {
        let requirement = self.normalize_builtin(facts, requirement)?;
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
