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
pub const trait Family{type Item0;type Item1;type Item2;type Item3;type Item4;type Item5;type Item6;type Item7;fn consume(_x0:Self::Item0,_x1:Self::Item1,_x2:Self::Item2,_x3:Self::Item3,_x4:Self::Item4,_x5:Self::Item5,_x6:Self::Item6,_x7:Self::Item7)where Self::Item0:[const]Destruct,Self::Item1:[const]Destruct,Self::Item2:[const]Destruct,Self::Item3:[const]Destruct,Self::Item4:[const]Destruct,Self::Item5:[const]Destruct,Self::Item6:[const]Destruct,Self::Item7:[const]Destruct{}}
impl Family for u8{type Item0=u8;type Item1=u8;type Item2=u8;type Item3=u8;type Item4=u8;type Item5=u8;type Item6=u8;type Item7=u8;}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Family>::consume(7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8);7}
