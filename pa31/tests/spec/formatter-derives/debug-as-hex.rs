/// Specifies whether the [`Debug`] trait should use lower-/upper-case
/// hexadecimal or normal integers.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[unstable(feature = "formatting_options", issue = "118117")]
pub enum DebugAsHex {
    /// Use lower-case hexadecimal integers for the `Debug` trait (like [the `x?` type](../../std/fmt/index.html#formatting-traits)).
    Lower,
    /// Use upper-case hexadecimal integers for the `Debug` trait (like [the `X?` type](../../std/fmt/index.html#formatting-traits)).
    Upper,
}
