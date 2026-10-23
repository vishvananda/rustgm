//! One lazy pending language condition preserves ordinary search frame storage.
pub(super) enum Pending {
    Direct(crate::sema::traits::callables::Matched),
    Forward(super::forwarding::Pending),
    Unsize(super::unsizing::Pending),
}
