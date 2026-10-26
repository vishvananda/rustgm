//! A concrete call proves only the header of its actual defining member.
use super::{Declarations, Dispatcher, Obligations, selection};
use crate::sema::specialization::{BodyRequirements, ProofInputs, SelectedMethod};
use std::sync::Arc;

impl Dispatcher {
    pub(super) fn prove_inherited(
        &mut self,
        facts: &mut Declarations,
        selected: SelectedMethod,
    ) -> Result<(), String> {
        let Some(mut requirements) = BodyRequirements::new(facts, selected)? else {
            return Ok(());
        };
        // Assumptions clone only its immutable index owner. The new Arc owns
        // this small shell; its bytes are paid before allocation.
        requirements.charge(1, size_of::<super::Assumptions>() + 2 * size_of::<usize>())?;
        let caller = self.caller.as_deref();
        let assumptions = caller.map_or_else(super::Assumptions::default, |caller| {
            caller.assumptions.clone()
        });
        let mut proof = Obligations::new(self.obligations.regions.end);
        let mut inputs = ProofInputs {
            frontier: proof.regions.end,
            caller: selection::OwnedCaller {
                assumptions: Arc::new(assumptions),
                environment: caller.and_then(|caller| caller.environment.clone()),
                first_region: proof.regions.end,
                retain_sized: false,
            },
            context: &mut self.context,
            proof: &mut proof,
            availability: self.availability,
        };
        if requirements.advance(facts, &mut inputs)?.is_some() {
            return Err("inherited dispatch requires pending defining-header preparation".into());
        }
        self.extend_evidence(proof)?;
        Ok(())
    }
}
