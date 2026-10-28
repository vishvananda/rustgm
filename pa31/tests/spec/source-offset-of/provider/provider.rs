#![feature(no_core,lang_items,rustc_attrs,intrinsics,builtin_syntax,offset_of_enum,decl_macro,allow_internal_unstable,staged_api)]
#![no_core]
#![unstable(feature="offset_provider",issue="none")]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_nounwind]
#[unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_const_unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
#[lang = "offset_of"]
pub const fn offset_of<T: PointeeSized>(variant: u32, field: u32) -> usize;
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

#[lang="Option"]pub enum Option<T>{#[lang="None"]None,#[lang="Some"]Some(T)}
#[repr(C)]pub struct Fields{pub first:u8,pub second:u64,pub third:u16}
fn payload<B>()->usize{offset_of!(Option<B>,Some.0)}
#[unsafe(no_mangle)]pub extern "C" fn option_u8()->usize{payload::<u8>()}
#[unsafe(no_mangle)]pub extern "C" fn option_u32()->usize{payload::<u32>()}
#[unsafe(no_mangle)]pub extern "C" fn option_u64()->usize{payload::<u64>()}
#[unsafe(no_mangle)]pub extern "C" fn option_ref()->usize{payload::<&'static u8>()}
#[unsafe(no_mangle)]pub extern "C" fn field_first()->usize{offset_of!(Fields,first)}
#[unsafe(no_mangle)]pub extern "C" fn field_second()->usize{offset_of!(Fields,second)}
#[unsafe(no_mangle)]pub extern "C" fn field_third()->usize{offset_of!(Fields,third)}
