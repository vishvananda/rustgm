//! Nominal use requirements are instantiated once; proofs belong to the caller's environment.
mod traits;
use super::model::Declarations;
use crate::{
    resolve::model::DefId,
    types::{
        GenericArgs, ItemId, Kind, TypeId,
        arguments::Argument,
        parameters::{
            ParameterKind,
            predicates::{Bound, Predicate},
        },
    },
};
use std::collections::HashSet;

/// `checked` belongs to one immutable caller environment. Cache only immediate
/// instance predicates; flattening every structural root would retain quadratic prefixes.
pub(super) fn requirements(
    facts: &mut Declarations,
    root: TypeId,
    checked: &mut HashSet<TypeId>,
) -> Result<Vec<Predicate>, &'static str> {
    let mut work = vec![root];
    let mut predicates = Vec::new();
    let mut unique = HashSet::new();
    while let Some(ty) = work.pop() {
        if !checked.insert(ty) {
            continue;
        }
        if checked
            .len()
            .saturating_add(work.len())
            .saturating_add(predicates.len())
            > 1_000_000
        {
            return Err("nominal requirement work budget exceeded");
        }
        match facts
            .types
            .kind(ty)
            .ok_or("invalid nominal requirement type")?
        {
            Kind::Adt(item, arguments) => {
                if !facts.generics.nominal_requirements.contains_key(&ty) {
                    let mut immediate = Vec::new();
                    instance(facts, item, arguments, &mut immediate)?;
                    facts
                        .generics
                        .nominal_requirements
                        .insert(ty, immediate.into());
                }
                for &predicate in facts.generics.nominal_requirements[&ty].iter() {
                    if unique.insert(predicate) {
                        predicates.push(predicate);
                    }
                }
                arguments_work(facts, arguments, &mut work)?;
            }
            Kind::Tuple(list) => {
                work.extend_from_slice(
                    facts
                        .types
                        .list(list)
                        .ok_or("invalid nominal requirement tuple")?,
                );
            }
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element) => work.push(element),
            Kind::Reference { referent, .. } => work.push(referent),
            Kind::Pointer { pointee, .. } => work.push(pointee),
            Kind::Function {
                parameters, result, ..
            } => {
                work.push(result);
                work.extend_from_slice(
                    facts
                        .types
                        .list(parameters)
                        .ok_or("invalid nominal requirement signature")?,
                );
            }
            Kind::FunctionItem { arguments, .. } => {
                // The declaration already proved its signature, including local
                // late binders. Item-use obligations validate its substitutions;
                // replaying the template signature would capture caller lifetimes.
                arguments_work(facts, arguments, &mut work)?;
            }
            _ => {}
        }
    }
    Ok(predicates)
}
fn arguments_work(
    facts: &Declarations,
    arguments: GenericArgs,
    work: &mut Vec<TypeId>,
) -> Result<(), &'static str> {
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("missing nominal requirement arguments")?;
    if work.len().saturating_add(arguments.len()) > 1_000_000 {
        return Err("nominal requirement pending work budget exceeded");
    }
    for argument in arguments {
        if let Argument::Type(ty) = argument {
            work.push(ty);
        }
    }
    Ok(())
}
fn instance(
    facts: &mut Declarations,
    item: ItemId,
    arguments: GenericArgs,
    output: &mut Vec<Predicate>,
) -> Result<(), &'static str> {
    let owner = DefId(item.0);
    if !facts.generics.nominal_contracts.contains(&owner) {
        return Err("nominal declaration requirements are not complete");
    }
    if let Some(parameters) = facts.generics.parameters.get(&owner) {
        let actual = facts
            .types
            .arguments(arguments)
            .ok_or("invalid nominal requirement arguments")?;
        if actual.len() != parameters.len() {
            return Err("nominal requirement argument arity mismatch");
        }
        for (parameter, argument) in parameters.iter().zip(actual) {
            if let Argument::Type(template) = parameter.argument {
                let Some(Kind::Parameter(id)) = facts.types.kind(template) else {
                    return Err("invalid nominal type parameter");
                };
                let Argument::Type(ty) = argument else {
                    return Err("nominal requirement argument kind mismatch");
                };
                if matches!(
                    facts.types.parameter(id).map(|p| p.kind),
                    Some(ParameterKind::Type { sized: true })
                ) {
                    output.push(Predicate {
                        ty,
                        bound: Bound::Sized,
                    });
                }
            }
        }
    }
    let predicates = facts
        .generics
        .predicates
        .get(&owner)
        .map_or(&[][..], Vec::as_slice);
    let count = predicates.len();
    for index in 0..count {
        let predicate = facts.generics.predicates[&owner][index].value;
        if matches!(predicate.bound, Bound::Outlives(_)) {
            continue;
        }
        let predicate =
            facts
                .generics
                .substitute_predicate(&mut facts.types, owner, arguments, predicate)?;
        output.push(predicate);
    }
    Ok(())
}

/// Predicate proofs are reusable only within one immutable declaration environment.
pub(super) struct Proof {
    environment: Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
    shapes: Option<crate::types::parameters::shape::Shapes>,
    copies: crate::types::copy::Copies,
    checked: HashSet<TypeId>,
    traits: Option<Box<traits::State>>,
    scope: Option<(u32, u32)>,
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
            checked: HashSet::new(),
            traits: None,
            scope: None,
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
            checked: HashSet::new(),
            traits: None,
            scope: None,
        }
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
            Bound::Trait(_) => self.prove_trait(facts, requirement)?,
            Bound::Sized => self.shape(facts, requirement.ty)?.sized,
            Bound::Copy | Bound::Clone => {
                facts.aggregates.prepare(&mut facts.types, requirement.ty)?;
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
