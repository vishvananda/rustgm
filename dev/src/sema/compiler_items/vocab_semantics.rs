//! Declaration recognition cannot silently grant an unimplemented compiler builtin.
use crate::{
    resolve::{bindings::Reservations, expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
};

pub(super) fn require_supported(report: &Report, reserved: &Reservations) -> Result<(), Error> {
    for role in LangItem::DECLARATIONS {
        if super::interval_values::NOMINALS.contains(&role) || role == LangItem::RangeInclusiveNew {
            continue;
        }
        if super::guard_panics::ROLES.contains(&role) {
            // Ordinary completed headers supply no compiler assertion routing.
            continue;
        }
        match role {
            // These roles complete through their actual ordinary declarations.
            // Generated state, implicit operations and executable calls still
            // require their separate typed contracts and ordinary body proofs.
            LangItem::CoroutineState
            | LangItem::TryTraitFromYeet
            | LangItem::CoercePointeeValidated
            | LangItem::ResumeTy
            | LangItem::GetContext
            | LangItem::Coroutine
            | LangItem::CoroutineYield
            | LangItem::CoroutineReturn
            | LangItem::CoroutineResume
            | LangItem::Poll
            | LangItem::PollReady
            | LangItem::PollPending
            | LangItem::Context
            | LangItem::VaArgSafe
            | LangItem::VaList
            | LangItem::CVoid
            | LangItem::Index
            | LangItem::IndexMut
            | LangItem::UnsafeCell
            | LangItem::UnsafePinned
            | LangItem::AddAssign
            | LangItem::SubAssign
            | LangItem::MulAssign
            | LangItem::DivAssign
            | LangItem::RemAssign
            | LangItem::BitXorAssign
            | LangItem::BitAndAssign
            | LangItem::BitOrAssign
            | LangItem::ShlAssign
            | LangItem::ShrAssign
            | LangItem::AsyncDropInPlace
            | LangItem::Mul
            | LangItem::Rem
            | LangItem::BitXor
            | LangItem::BitAnd
            | LangItem::Shr
            | LangItem::UseCloned
            | LangItem::FnPtrTrait
            | LangItem::FnPtrAddr
            | LangItem::Future
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
            | LangItem::FusedIterator
            | LangItem::AsyncIterator
            | LangItem::AsyncIteratorPollNext
            | LangItem::AsyncGenReady
            | LangItem::AsyncGenPending
            | LangItem::AsyncGenFinished
            | LangItem::IntoAsyncIterIntoIter
            | LangItem::IntoFutureIntoFuture
            | LangItem::Pin
            | LangItem::PinNewUnchecked
            | LangItem::Deref
            | LangItem::DerefMut
            | LangItem::DerefPure
            | LangItem::DerefTarget
            | LangItem::Receiver
            | LangItem::ReceiverTarget
            | LangItem::CompilerMove
            | LangItem::CompilerCopy => {}
            _ => {
                if let Some(declaration) = reserved.language(role) {
                    return Err(Declarations::error(
                        report,
                        declaration.origin,
                        "source compiler language item semantic contract is not implemented",
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/vocab_claims.rs"]
mod tests;
