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
pub const trait Batch<T0,T1,T2,T3,T4,T5,T6,T7>:Sized where T0:[const]Destruct,T1:[const]Destruct,T2:[const]Destruct,T3:[const]Destruct,T4:[const]Destruct,T5:[const]Destruct,T6:[const]Destruct,T7:[const]Destruct{fn consume(_x0:T0,_x1:T1,_x2:T2,_x3:T3,_x4:T4,_x5:T5,_x6:T6,_x7:T7){}}
impl<T0,T1,T2,T3,T4,T5,T6,T7> Batch<T0,T1,T2,T3,T4,T5,T6,T7> for u8{}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Batch<u8,u8,u8,u8,u8,u8,u8,u8>>::consume(7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8);7}
