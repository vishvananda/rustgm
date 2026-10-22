//! Source observers borrow one selection before its evidence joins the caller.
use super::{Declarations, Dispatcher, Obligations, arguments, implementation};
use crate::sema::values::Instance;

/// One exact selection before its evidence joins the caller's append-only
/// proof. A warm target is identity only; the observer must already possess a
/// seal for this actual caller/context or request a fresh checked selection.
#[derive(Clone, Copy)]
pub(in crate::sema) struct ConstantProof<'a> {
    pub input: Instance,
    pub selected: Instance,
    pub availability: crate::sema::traits::implementations::Availability,
    pub first: u32,
    pub proof: Option<&'a Obligations>,
}

pub(in crate::sema) enum ConstantDecision {
    Checked(Option<Obligations>),
    Fresh,
}

impl Dispatcher {
    pub(super) fn select_constant_using<const OBSERVE: bool>(
        &mut self,
        facts: &mut Declarations,
        instance: Instance,
        observer: &mut impl FnMut(
            &mut Declarations,
            ConstantProof<'_>,
        ) -> Result<ConstantDecision, String>,
    ) -> Result<Instance, String> {
        let cached = self.constants.get(&instance).copied();
        if let Some(selected) = cached
            && (!OBSERVE || self.observe_constant(facts, instance, selected, None, observer)?)
        {
            return Ok(selected);
        }
        let Some(owner) = facts.associated.trait_owner(instance.definition) else {
            if OBSERVE {
                let proof = Obligations::new(self.obligations.regions.end);
                if !self.observe_constant(facts, instance, instance, Some(&proof), observer)? {
                    return Err("fresh constant selection requested another selection".into());
                }
            }
            return Ok(instance);
        };
        let arguments = arguments::Arguments::constant(facts, instance)?;
        let predicate = arguments.goal(facts, owner)?;
        if self.caller.is_none() && predicate.contains_parameters(&facts.types)? {
            return Err("concrete constant retains unresolved arguments".into());
        }
        let (owner, arguments, evidence) = self.implementation(facts, predicate)?;
        let definition = facts
            .generics
            .implementations
            .as_ref()
            .and_then(|registry| {
                registry.available_constant(owner, instance.definition, self.availability)
            })
            .ok_or("selected implementation lacks the trait constant")?;
        let result = if definition == instance.definition {
            instance
        } else {
            Instance {
                definition,
                arguments,
            }
        };
        let result = implementation::restore_constant(
            facts,
            &mut self.context,
            result,
            &evidence,
            &self.obligations,
        )?;
        // The observer can borrow this exact payload before it is moved. Its
        // returned debts begin after this selection, preserving append order.
        if cached.is_some_and(|previous| previous != result) {
            return Err("source constant selection changed its cached target".into());
        }
        let added = if OBSERVE {
            let decision = observer(
                facts,
                ConstantProof {
                    input: instance,
                    selected: result,
                    availability: self.availability,
                    first: evidence.regions.end,
                    proof: Some(&evidence),
                },
            )?;
            match decision {
                ConstantDecision::Checked(proof) => proof,
                ConstantDecision::Fresh => {
                    return Err("fresh constant selection requested another selection".into());
                }
            }
        } else {
            None
        };
        self.extend_evidence(evidence)?;
        if let Some(added) = added {
            self.extend_evidence(added)?;
        }
        self.constants.insert(instance, result);
        Ok(result)
    }
    fn observe_constant(
        &mut self,
        facts: &mut Declarations,
        input: Instance,
        selected: Instance,
        proof: Option<&Obligations>,
        observer: &mut impl FnMut(
            &mut Declarations,
            ConstantProof<'_>,
        ) -> Result<ConstantDecision, String>,
    ) -> Result<bool, String> {
        let decision = observer(
            facts,
            ConstantProof {
                input,
                selected,
                availability: self.availability,
                first: self.obligations.regions.end,
                proof,
            },
        )?;
        match decision {
            ConstantDecision::Checked(added) => {
                if let Some(added) = added {
                    self.extend_evidence(added)?;
                }
                Ok(true)
            }
            ConstantDecision::Fresh => Ok(false),
        }
    }
    /// The immutable initial binder is a goal payload, not an assumption. The
    /// exact actual caller header supplies its count; retained source intervals
    /// and unrelated appended debts do not participate in this bounded read.
    pub(in crate::sema) fn source_opening(
        &self,
        universals: u32,
    ) -> Result<Obligations, &'static str> {
        let first = self.obligations.regions.start;
        let end = first
            .checked_add(universals)
            .ok_or("source opening overflow")?;
        let pairs = self
            .obligations
            .equalities
            .get(..universals as usize)
            .ok_or("missing source opening")?;
        for (slot, &(left, right)) in pairs.iter().enumerate() {
            let slot = u16::try_from(slot).map_err(|_| "source opening arity exhausted")?;
            if left != crate::types::RegionId(first + u32::from(slot))
                || right != crate::types::RegionId::bound(0, slot)?
            {
                return Err("source opening differs from the actual caller binder");
            }
        }
        let mut result = Obligations::new(first);
        result.regions.end = end;
        result.equalities.extend_from_slice(pairs);
        Ok(result)
    }
}
