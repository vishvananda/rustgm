//! Executable cleanup consumes immutable phase-neutral data, never a synthesized Call.
mod codebind;
mod ctsites;
mod drive;
mod returned;
#[cfg(test)]
#[path = "../../tests/invariants/ctgate.rs"]
pub(in crate::const_eval) mod tests;
use crate::{
    body::{
        drops::schedule::{Call as Method, View},
        ir::{Body, BodyKey, BodyOrigin},
        regions::Contract,
    },
    sema::model::Declarations,
    types::{TypeId, arena::Identity},
};
pub(super) use drive::Step;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    sync::Arc,
};

pub(crate) struct Bundle {
    view: View,
    contract: Arc<Contract>,
    key: BodyKey,
    source: BodyOrigin,
    signatures: Option<(TypeId, TypeId)>,
    result: TypeId,
    types: Identity,
    definitions: Identity,
    binding: RefCell<Option<Weak<super::code::Code>>>,
}
impl Bundle {
    pub(crate) fn allocation_bytes(&self) -> usize {
        crate::support::retained::sum([
            size_of::<Self>() + 2 * size_of::<usize>(),
            self.view.allocation_bytes(),
            self.contract.retained_bytes() + 2 * size_of::<usize>(),
        ])
    }
    fn state(&self) -> Result<&crate::body::defining_cleanups::State, &'static str> {
        self.contract
            .builtins
            .as_deref()
            .and_then(|b| b.defining())
            .ok_or("executable cleanup lost its actual defining State")
    }
}
pub(crate) fn prepare(
    facts: &Declarations,
    body: &Body,
    contract: &Contract,
    view: View,
    budget: usize,
    work: &mut usize,
) -> Result<Rc<Bundle>, &'static str> {
    let mut proof_work = crate::body::defining_cleanups::Work::new(1_000_000);
    let charge = crate::body::copy_requirements::OriginalIndexBudget::new(1_000_000, budget);
    let result = (|| {
        let state = contract
            .builtins
            .as_deref()
            .and_then(|b| b.defining())
            .ok_or("executable cleanup requires its actual defining State")?;
        state.checked(
            body,
            contract
                .builtins
                .as_deref()
                .and_then(|b| b.environment.as_ref()),
            &mut proof_work,
        )?;
        let fixed = size_of::<Bundle>()
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(view.allocation_bytes())
            .saturating_add(contract.retained_bytes())
            .saturating_add(2 * size_of::<usize>());
        // Code binding repeats immutable completeness/local checks at publication.
        charge.charge(
            contract
                .retained_bytes()
                .div_ceil(size_of::<usize>())
                .saturating_add(view.active.len())
                .saturating_add(view.calls.len().saturating_mul(10))
                .saturating_add(32),
            fixed,
        )?;
        Ok(Rc::new(Bundle {
            view,
            contract: Arc::new(contract.clone()),
            key: body.key(),
            source: body.source,
            signatures: body.owner.signatures(),
            result: body.result,
            types: facts
                .types
                .existing_arena_identity()
                .cloned()
                .ok_or("cleanup lost its type arena")?,
            definitions: facts
                .aggregates
                .existing_arena_identity()
                .cloned()
                .ok_or("cleanup lost its definition arena")?,
            binding: RefCell::new(None),
        }))
    })();
    *work = work
        .saturating_add(proof_work.steps)
        .saturating_add(charge.costs().0);
    result
}
