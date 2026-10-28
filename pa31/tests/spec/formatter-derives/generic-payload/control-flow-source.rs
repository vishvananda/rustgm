#[stable(feature = "control_flow_enum_type", since = "1.55.0")]
#[rustc_diagnostic_item = "ControlFlow"]
#[must_use]
// ControlFlow should not implement PartialOrd or Ord, per RFC 3058:
// https://rust-lang.github.io/rfcs/3058-try-trait-v2.html#traits-for-controlflow
#[derive(Copy, Debug, Hash)]
#[derive_const(Clone, PartialEq, Eq)]
pub enum ControlFlow<B, C = ()> {
    /// Move on to the next phase of the operation as normal.
    #[stable(feature = "control_flow_enum_type", since = "1.55.0")]
    #[lang = "Continue"]
    Continue(C),
    /// Exit the operation without running subsequent phases.
    #[stable(feature = "control_flow_enum_type", since = "1.55.0")]
    #[lang = "Break"]
    Break(B),
    // Yes, the order of the variants doesn't match the type parameters.
    // They're in this order so that `ControlFlow<A, B>` <-> `Result<B, A>`
    // is a no-op conversion in the `Try` implementation.
}
