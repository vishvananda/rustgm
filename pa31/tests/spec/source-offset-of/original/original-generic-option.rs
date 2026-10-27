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
fn payload<B>()->usize{offset_of!(Option<B>,Some.0)}
fn main(){println!("{} {} {} {}",payload::<u8>(),payload::<u32>(),payload::<u64>(),payload::<&u8>());assert!(payload::<u32>()>0);}
