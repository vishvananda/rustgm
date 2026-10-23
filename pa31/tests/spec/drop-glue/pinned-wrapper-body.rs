#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl,auto_traits)]
#![no_core]
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
pub mod intrinsics{use crate::Sized;#[rustc_intrinsic]pub const fn needs_drop<T:?Sized>()->bool;}
pub const fn needs_drop<T: ?Sized>() -> bool {
    const { intrinsics::needs_drop::<T>() }
}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang = "drop_glue"]
pub(crate) const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
}

pub const unsafe fn drop_in_place<T: PointeeSized>(to_drop: *mut T)
where
    T: [const] Destruct,
{
    // Due to historic reasons, `drop_in_place` takes a pointer rather than a reference,
    // which results in worse codegen since we don't apply noalias/dereferenceable llvm
    // attributes to pointer arguments. To workaround this without breaking public
    // interface, `drop_in_place` calls the lang item, rather than being one directly.

    // SAFETY:
    // - compiler glue has the same safety requirements as this function
    // - the pointer must be valid as per the safety requirement of this function
    unsafe { drop_glue(&mut *to_drop) }
}
#[repr(C)]pub struct D<'a>(&'a mut u8);impl<'a> Drop for D<'a>{fn drop(&mut self){*self.0=7;}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn observe(value:&mut D<'_>){unsafe{drop_in_place(value as *mut D<'_>);}}
