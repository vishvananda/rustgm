#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl,const_destruct,auto_traits)]
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

struct D<'a>(&'a mut u8);
impl<'a> Drop for D<'a>{fn drop(&mut self){*self.0=7;}}
pub const trait Family{type Item0;fn consume(_x0:Self::Item0,_x1:Self::Item0,_x2:Self::Item0,_x3:Self::Item0,_x4:Self::Item0,_x5:Self::Item0,_x6:Self::Item0,_x7:Self::Item0,_x8:Self::Item0,_x9:Self::Item0,_x10:Self::Item0,_x11:Self::Item0,_x12:Self::Item0,_x13:Self::Item0,_x14:Self::Item0,_x15:Self::Item0,_x16:Self::Item0,_x17:Self::Item0,_x18:Self::Item0,_x19:Self::Item0,_x20:Self::Item0,_x21:Self::Item0,_x22:Self::Item0,_x23:Self::Item0,_x24:Self::Item0,_x25:Self::Item0,_x26:Self::Item0,_x27:Self::Item0,_x28:Self::Item0,_x29:Self::Item0,_x30:Self::Item0,_x31:Self::Item0)where Self::Item0:[const]Destruct{}}
impl Family for u8{type Item0=u8;}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Family>::consume(7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8);7}
