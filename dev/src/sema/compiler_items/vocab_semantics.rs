//! Declaration recognition cannot silently grant an unimplemented compiler builtin.
use crate::{
    resolve::{bindings::Reservations, expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
};

pub(super) fn require_supported(report: &Report, reserved: &Reservations) -> Result<(), Error> {
    for role in LangItem::DECLARATIONS {
        if matches!(
            role,
            LangItem::Poll | LangItem::PollReady | LangItem::PollPending | LangItem::Context
        ) {
            // Actual nominal/variant producers complete after their ordinary fields.
            // The demanded polling protocol has separate signature/ownership checks.
            continue;
        }
        if matches!(
            role,
            LangItem::VaArgSafe | LangItem::VaList | LangItem::CVoid
        ) {
            // Actual ordinary trait/nominal producers retain their headers and
            // policies. Variadic initialization and reads need their own proof.
            continue;
        }
        if matches!(role, LangItem::Index | LangItem::IndexMut) {
            // Ordinary headers supply no operation permission: actual indexing
            // still needs the checked method, parent/output and normal call proof.
            continue;
        }
        if matches!(role, LangItem::UnsafeCell | LangItem::UnsafePinned) {
            // The authenticated nominal producer installs interior storage with
            // the actual shape. Ordinary trait and body proofs remain required.
            continue;
        }
        if matches!(
            role,
            LangItem::AddAssign
                | LangItem::SubAssign
                | LangItem::MulAssign
                | LangItem::DivAssign
                | LangItem::RemAssign
                | LangItem::BitXorAssign
                | LangItem::BitAndAssign
                | LangItem::BitOrAssign
                | LangItem::ShlAssign
                | LangItem::ShrAssign
        ) {
            // Complete ordinary declarations; demanded mutation still requires
            // its exact source header and the existing mutable-call proof.
            continue;
        }
        if role == LangItem::AsyncDropInPlace {
            // Authenticate its ordinary source declaration; generated future
            // storage and cleanup execution have a separate demanded boundary.
            continue;
        }
        if matches!(
            role,
            LangItem::Mul | LangItem::Rem | LangItem::BitXor | LangItem::BitAnd | LangItem::Shr
        ) {
            // Actual ordinary headers complete here. A demanded operation still
            // requires the shared operator contract and normal typed call proof.
            continue;
        }
        if matches!(
            role,
            LangItem::UseCloned | LangItem::FnPtrTrait | LangItem::FnPtrAddr
        ) {
            // Reserve the actual ordinary trait, then require its canonical header
            // completion in roles::validate. This supplies no implementation proof
            // or implicit use/clone operation; those have separate typed consumers.
            continue;
        }
        if matches!(
            role,
            LangItem::Future
                | LangItem::FutureOutput
                | LangItem::FuturePoll
                | LangItem::AsyncDrop
                | LangItem::AsyncFn
                | LangItem::AsyncFnMut
                | LangItem::AsyncFnOnce
                | LangItem::AsyncFnOnceOutput
                | LangItem::CallOnceFuture
                | LangItem::CallRefFuture
                | LangItem::AsyncFnKindHelper
                | LangItem::AsyncFnKindUpvars
        ) {
            // Genuine ordinary declarations are completed by the Future producer.
            // Polling and future state machines remain separate demanded consumers.
            continue;
        }
        if matches!(role, LangItem::Pin | LangItem::PinNewUnchecked) {
            // Actual nominal/member completion is separate from pinned receivers,
            // implicit pin construction and async state-machine permission.
            continue;
        }
        if matches!(
            role,
            LangItem::Deref
                | LangItem::DerefMut
                | LangItem::DerefPure
                | LangItem::DerefTarget
                | LangItem::Receiver
                | LangItem::ReceiverTarget
        ) {
            // Canonical source owners still complete below; dereference use
            // requires ordinary trait selection and the actual method contract.
            continue;
        }
        if matches!(role, LangItem::CompilerMove | LangItem::CompilerCopy) {
            // These hooks keep their actual ordinary bodies. Current codegen emits no
            // profiling annotations; an invoked hook must still execute its source body.
            continue;
        }
        if let Some(declaration) = reserved.language(role) {
            return Err(Declarations::error(
                report,
                declaration.origin,
                "source compiler language item semantic contract is not implemented",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/vocab_claims.rs"]
mod tests;
