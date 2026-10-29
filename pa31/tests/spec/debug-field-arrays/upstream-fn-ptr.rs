#[derive(Debug)]
#[unstable(feature = "type_info", issue = "146922")]
/// Function pointer, e.g. fn(u8),
pub struct FnPtr {
    /// Unsafety, true is unsafe
    pub unsafety: bool,

    /// Abi, e.g. extern "C"
    pub abi: Abi,

    /// Function inputs
    pub inputs: &'static [TypeId],

    /// Function return type, default is TypeId::of::<()>
    pub output: TypeId,

    /// Vardiadic function, e.g. extern "C" fn add(n: usize, mut args: ...);
    pub variadic: bool,

    // FIXME(splat): should these fields be private, or merged into an Option<u8/u16>?
    /// Is any function argument splatted?
    pub is_splatted: bool,

    /// The index of the splatted function argument in `inputs`, only valid if `is_splatted` is true.
    /// e.g. in `fn overload(a: u8, #[rustc_splat] b: (f32, usize))` the index is 1, and it can be called
    /// as `overload(a, 1.0, 2)`.
    pub splatted_index: u8,
}
