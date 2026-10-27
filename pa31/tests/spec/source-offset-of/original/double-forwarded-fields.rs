#![feature(staged_api,decl_macro,allow_internal_unstable,rustc_attrs,offset_of_enum)]
#![unstable(feature="offset_fixture",issue="none")]
#[stable(feature = "offset_of", since = "1.77.0")]
#[diagnostic::on_unmatched_args(
    note = "this macro expects a container type and a (nested) field path, like `offset_of!(Type, field)`"
)]
#[doc(alias = "memoffset")]
#[allow_internal_unstable(builtin_syntax, core_intrinsics)]
pub macro offset_of($Container:ty, $($fields:expr)+ $(,)?) {
    // The `{}` is for better error messages
    const {builtin # offset_of($Container, $($fields)+)}
}
macro_rules! f{($t:ty,$e:expr)=>{offset_of!($t,$e)}}macro_rules! g{($t:ty,$e:expr)=>{f!($t,$e)}}fn payload<T>()->usize{g!(Option<T>,Some.0)}fn use_it()->usize{payload::<u64>()}
