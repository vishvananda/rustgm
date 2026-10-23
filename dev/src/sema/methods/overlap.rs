//! Isolated existential header equations. These proofs never authorize body operations.
use crate::{
    resolve::model::DefId,
    sema::{model::Declarations, nominals::Proof},
    types::{
        arguments::Argument as Canonical,
        infer::{Context, Domain, Error, Instantiation, TermId, arguments::Argument},
        parameters::{
            ParameterId, ParameterKind,
            environment::{Abilities, Environment},
            predicates::{Bound, Predicate},
        },
    },
};
use std::sync::Arc;

#[derive(Default)]
struct Probe {
    context: Context,
    bindings: Vec<(ParameterId, Argument, Canonical)>,
    obligations: Vec<(TermId, Bound)>,
    trait_obligations: bool,
}

pub(in crate::sema) fn possible(
    facts: &mut Declarations,
    left: DefId,
    right: DefId,
) -> Result<(bool, usize), &'static str> {
    let mut probe = Probe::default();
    let left = probe.header(facts, left)?;
    let right = probe.header(facts, right)?;
    let possible = match probe.context.equate_shape(&facts.types, left, right) {
        Ok(()) => {
            let possible = probe.obligations(facts)?;
            if possible && probe.trait_obligations {
                return Err("user trait overlap obligations require trait coherence");
            }
            possible
        }
        Err(Error::Mismatch | Error::Occurs) => false,
        Err(error) => return Err(inference_error(error)),
    };
    Ok((possible, probe.context.steps))
}

impl Probe {
    fn header(&mut self, facts: &mut Declarations, owner: DefId) -> Result<TermId, &'static str> {
        let target = facts
            .associated
            .schema(owner)
            .ok_or("missing overlap owner schema")?
            .self_type;
        let start = self.bindings.len();
        let mut bindings = Vec::new();
        for parameter in facts.generics.parameters.get(&owner).into_iter().flatten() {
            let (id, argument) = match parameter.argument {
                Canonical::Lifetime(_) => continue,
                Canonical::Type(ty) => {
                    let Some(crate::types::Kind::Parameter(id)) = facts.types.kind(ty) else {
                        return Err("invalid overlap type parameter");
                    };
                    let term = self
                        .context
                        .variable(Domain::Any)
                        .map_err(inference_error)?;
                    (id, Argument::Type(term))
                }
                Canonical::Const(value) => {
                    let Some(crate::types::arguments::ConstKind::Parameter(id)) =
                        facts.types.const_kind(value)
                    else {
                        return Err("invalid overlap const parameter");
                    };
                    let Some(ParameterKind::Const { ty }) =
                        facts.types.parameter(id).map(|p| p.kind)
                    else {
                        return Err("missing overlap const parameter type");
                    };
                    let term = self
                        .context
                        .const_variable(&facts.types, ty)
                        .map_err(inference_error)?;
                    (id, Argument::Const(term))
                }
            };
            bindings.push((id, argument));
            self.bindings.push((id, argument, parameter.argument));
        }
        let mut substitution = Instantiation::new(&mut self.context, &facts.types, &bindings)
            .map_err(inference_error)?;
        let target = self
            .context
            .instantiate_with(&mut facts.types, target, &mut substitution)
            .map_err(inference_error)?;
        for &(id, argument, _) in &self.bindings[start..] {
            if let Argument::Type(term) = argument
                && matches!(
                    facts.types.parameter(id).map(|p| p.kind),
                    Some(ParameterKind::Type { sized: true })
                )
            {
                self.obligations.push((term, Bound::Sized));
            }
        }
        for predicate in facts.generics.predicates_in_scope(owner) {
            let predicate = predicate.value.ordinary()?;
            if matches!(
                predicate.bound,
                Bound::Trait(_)
                    | Bound::ConstTrait(_)
                    | Bound::MaybeConstTrait(_)
                    | Bound::ProjectionEqual(_)
            ) {
                // Structurally disjoint headers need no trait coherence proof.
                self.trait_obligations = true;
                continue;
            }
            if matches!(predicate.bound, Bound::Outlives(_)) {
                continue;
            }
            let term = self
                .context
                .instantiate_with(&mut facts.types, predicate.ty, &mut substitution)
                .map_err(inference_error)?;
            self.obligations.push((term, predicate.bound));
        }
        Ok(target)
    }

    fn obligations(&mut self, facts: &mut Declarations) -> Result<bool, &'static str> {
        // Preserve unification equivalence classes. A remaining existential has a
        // representative parameter with every positive built-in ability available.
        // This is a possibility environment, not the generic body's environment.
        let mut assumptions = Vec::new();
        for &(id, argument, canonical) in &self.bindings {
            match (argument, canonical) {
                (Argument::Type(term), Canonical::Type(ty)) => {
                    if self
                        .context
                        .head(&facts.types, term)
                        .map_err(inference_error)?
                        .is_none()
                    {
                        let rigid = self
                            .context
                            .known(&facts.types, ty)
                            .map_err(inference_error)?;
                        self.context
                            .equate_shape(&facts.types, term, rigid)
                            .map_err(inference_error)?;
                        assumptions.push((
                            id,
                            Abilities {
                                copy: true,
                                clone_: true,
                            },
                        ));
                    }
                }
                (Argument::Const(term), Canonical::Const(value)) => {
                    match self.context.resolved_const(&mut facts.types, term) {
                        Ok(_) => {}
                        Err(Error::Unresolved) => {
                            let rigid = self
                                .context
                                .known_const(&facts.types, value)
                                .map_err(inference_error)?;
                            self.context
                                .equate_consts(term, rigid)
                                .map_err(inference_error)?;
                        }
                        Err(error) => return Err(inference_error(error)),
                    }
                }
                _ => return Err("overlap argument kind mismatch"),
            }
        }
        let environment = Arc::new(Environment::new(&facts.types, &assumptions)?);
        let mut proof = Proof::for_coherence(environment);
        for &(term, bound) in &self.obligations {
            let ty = self
                .context
                .resolved(&mut facts.types, term)
                .map_err(inference_error)?;
            if !proof.prove(facts, Predicate { ty, bound })? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn inference_error(error: Error) -> &'static str {
    match error {
        Error::InvalidIdentity => "invalid overlap inference identity",
        Error::Overflow => "overlap inference arena exceeds 32 bits",
        Error::Mismatch => "inconsistent overlap equation",
        Error::Occurs => "recursive overlap equation",
        Error::Unresolved => "unresolved overlap parameter",
        Error::Budget => "overlap inference work budget exceeded",
        Error::Normalization => "overlap projection requires normalization",
    }
}
