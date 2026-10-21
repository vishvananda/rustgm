//! Shared typed entry selection precedes source proof, runtime erasure and lowering.
use super::{Entry, borrowed, matching, require_conditions};
use crate::{
    body::regions::{Contract, Kind, Obligation},
    resolve::unit::NodeKey,
    sema::{closures::Body, model::Declarations},
    support::call_traits::CallTrait,
    types::{
        Kind as Type, Store, TypeId, callable::Convention, closure::Adapter, instance::Instance,
    },
};
pub(super) struct Candidate {
    source: Body,
    instance: Instance,
    adapter: Option<Adapter>,
    receiver: TypeId,
    requested: CallTrait,
}
impl Candidate {
    pub(super) fn select(
        facts: &Declarations,
        receiver: TypeId,
        kind: CallTrait,
    ) -> Result<Option<Self>, &'static str> {
        let Some((subject, ty)) = captured(&facts.types, receiver, kind)? else {
            return Ok(None);
        };
        let Some(Type::Closure { identity, .. }) = facts.types.kind(ty) else {
            return Err("capture leaf is not a closure");
        };
        let source = *facts
            .closures
            .identity(identity)
            .ok_or("missing captured closure source")?;
        let instance = Instance::closure(&facts.types, ty)?;
        let adapter = borrowed::adapter(&facts.types, instance, subject, kind)?;
        Ok(Some(Self {
            source,
            instance,
            adapter,
            receiver,
            requested: kind,
        }))
    }
    pub(super) fn finish(
        self,
        facts: &mut Declarations,
        contract: &mut Contract,
        caller: NodeKey,
        selected: TypeId,
        proofs: &mut crate::body::instantiate::ProofCursor,
    ) -> Result<Entry, &'static str> {
        validate_opened(&facts.types, contract, caller, self.receiver, selected)?;
        let Self {
            source,
            instance,
            adapter,
            requested: kind,
            ..
        } = self;
        let identity = source.identity;
        let generic = source
            .owner
            .is_some_and(|owner| facts.generics.has_parameters(owner));
        let super::shaping::Prepared {
            execution,
            mut equalities,
            proofs: retained,
            ..
        } = super::shaping::prepare(facts, source, instance, adapter)?;
        let bindings = matching::execution_bindings(&facts.types, execution, selected)?;
        for proof in retained {
            proofs.retain(
                facts,
                contract,
                proof,
                bindings.slots.iter().map(|(&slot, &actual)| (slot, actual)),
            )?;
        }
        if generic {
            require_conditions(facts, identity, &bindings, caller, contract)?;
        }
        equalities.extend(bindings.equalities);
        contract
            .obligations
            .extend(equalities.into_iter().map(|pair| Obligation {
                source: caller,
                kind: Kind::Equality(pair),
            }));
        Ok(Entry {
            instance,
            requested: kind,
            execution: selected,
            adapter,
        })
    }
}
/// Selected signatures have already opened their own call binder in the caller scope.
/// This helper emits debts; exact source and wrapper proofs remain mandatory.
pub(crate) fn select_execution(
    facts: &mut Declarations,
    input: &mut Contract,
    caller: NodeKey,
    requested: CallTrait,
    receiver: TypeId,
    selected: TypeId,
    proofs: &mut crate::body::instantiate::ProofCursor,
) -> Result<Option<Entry>, &'static str> {
    if let Some(candidate) = Candidate::select(facts, receiver, requested)? {
        candidate
            .finish(facts, input, caller, selected, proofs)
            .map(Some)
    } else {
        validate_opened(&facts.types, input, caller, receiver, selected)?;
        Ok(None)
    }
}
fn validate_opened(
    types: &Store,
    input: &Contract,
    caller: NodeKey,
    receiver: TypeId,
    selected: TypeId,
) -> Result<(), &'static str> {
    if caller != input.source {
        return Err("closure entry caller differs from its contract owner");
    }
    let Some(Type::Function {
        bound_regions: 0,
        unsafe_: false,
        convention,
        parameters,
        ..
    }) = types.kind(selected)
    else {
        return Err("closure entry requires an opened safe Rust signature");
    };
    if convention != Convention::RUST
        || types.list(parameters).and_then(|p| p.first()).copied() != Some(receiver)
    {
        return Err("closure entry signature differs from its actual receiver or convention");
    }
    Ok(())
}

pub(super) fn captured(
    types: &Store,
    receiver: TypeId,
    requested: CallTrait,
) -> Result<Option<(TypeId, TypeId)>, &'static str> {
    let subject = borrowed::subject(types, receiver, requested)?;
    let Some(ty) = borrowed::leaf(types, subject)? else {
        return Ok(None);
    };
    Ok(
        matches!(types.kind(ty),Some(Type::Closure {environment,..}) if environment!=TypeId::UNIT)
            .then_some((subject, ty)),
    )
}
