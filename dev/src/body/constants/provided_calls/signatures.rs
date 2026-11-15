//! Unsafe defining requests consume exact discharged call safety, never spelling.
use super::{Body, Call, Declarations, Kind, Work};
use crate::{resolve::unit::NodeKey, support::crates::generics::source::CallSafety, types::TypeId};
#[derive(Clone, Copy)]
pub(super) struct Signature {
    target: TypeId,
    opened: TypeId,
    unsafe_: bool,
}
impl Signature {
    pub(super) fn permits(
        self,
        facts: &Declarations,
        target: TypeId,
        signature: TypeId,
        opened: bool,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        if target != self.target || (opened && signature != self.opened) {
            return Ok(false);
        }
        if !self.unsafe_ {
            return super::super::invocations::member_signature(
                facts, target, signature, opened, work,
            );
        }
        work.charge(1)?;
        Ok(matches!(facts.types.kind(signature), Some(Kind::Function {
            unsafe_: true, convention: crate::types::callable::Convention::RUST,
            bound_regions, ..
        }) if !opened || bound_regions == 0))
    }
}
pub(super) fn checked(
    facts: &Declarations,
    body: &Body,
    owner: NodeKey,
    call: Call,
    work: &mut Work,
) -> Result<Option<Signature>, &'static str> {
    let safe = super::super::invocations::member_signature(
        facts,
        call.target,
        call.signature,
        true,
        work,
    )?;
    let selected = Signature {
        target: call.target,
        opened: call.signature,
        unsafe_: !safe,
    };
    if safe {
        return Ok(Some(selected));
    }
    work.charge(2)?;
    if !matches!(
        facts.types.kind(call.signature),
        Some(Kind::Function {
            unsafe_: true,
            convention: crate::types::callable::Convention::RUST,
            bound_regions: 0,
            ..
        })
    ) {
        return Ok(None);
    }
    let Some(node) = facts.node(call.site) else {
        return Ok(None);
    };
    let Some(crate::sema::model::Operation::Call {
        signature,
        source_use: Some(source_use),
    }) = node.operation
    else {
        return Ok(None);
    };
    if signature != call.signature || node.ty != Some(call.result) {
        return Ok(None);
    }
    // At most one million safety rows: <=20 comparisons plus owner/index
    // lookups. Both successful and missing rows pay the same upper bound.
    work.charge(32)?;
    match source_use.call_safety(facts, owner, call.site.node, signature)? {
        CallSafety::ExplicitUnsafeBlock => Ok(Some(selected)),
        CallSafety::UnsafeFunction => {
            work.charge(1)?;
            Ok(body
                .owner
                .signatures()
                .filter(|(public, _)| {
                    matches!(
                        facts.types.kind(*public),
                        Some(Kind::Function { unsafe_: true, .. })
                    )
                })
                .map(|_| selected))
        }
        CallSafety::Safe => Ok(None),
    }
}
