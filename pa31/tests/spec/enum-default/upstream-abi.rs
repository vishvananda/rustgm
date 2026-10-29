#[derive(Debug, Default)]
#[non_exhaustive]
#[unstable(feature = "type_info", issue = "146922")]
/// Abi of [FnPtr]
pub enum Abi {
    /// Named abi, e.g. extern "custom", "stdcall" etc.
    Named(&'static str),

    /// Default
    #[default]
    ExternRust,

    /// C-calling convention
    ExternC,
}