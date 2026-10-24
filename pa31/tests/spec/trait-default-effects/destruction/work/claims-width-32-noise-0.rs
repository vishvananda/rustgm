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
pub const trait Batch<T0,T1,T2,T3,T4,T5,T6,T7,T8,T9,T10,T11,T12,T13,T14,T15,T16,T17,T18,T19,T20,T21,T22,T23,T24,T25,T26,T27,T28,T29,T30,T31>:Sized where T0:[const]Destruct,T1:[const]Destruct,T2:[const]Destruct,T3:[const]Destruct,T4:[const]Destruct,T5:[const]Destruct,T6:[const]Destruct,T7:[const]Destruct,T8:[const]Destruct,T9:[const]Destruct,T10:[const]Destruct,T11:[const]Destruct,T12:[const]Destruct,T13:[const]Destruct,T14:[const]Destruct,T15:[const]Destruct,T16:[const]Destruct,T17:[const]Destruct,T18:[const]Destruct,T19:[const]Destruct,T20:[const]Destruct,T21:[const]Destruct,T22:[const]Destruct,T23:[const]Destruct,T24:[const]Destruct,T25:[const]Destruct,T26:[const]Destruct,T27:[const]Destruct,T28:[const]Destruct,T29:[const]Destruct,T30:[const]Destruct,T31:[const]Destruct{fn consume(_x0:T0,_x1:T1,_x2:T2,_x3:T3,_x4:T4,_x5:T5,_x6:T6,_x7:T7,_x8:T8,_x9:T9,_x10:T10,_x11:T11,_x12:T12,_x13:T13,_x14:T14,_x15:T15,_x16:T16,_x17:T17,_x18:T18,_x19:T19,_x20:T20,_x21:T21,_x22:T22,_x23:T23,_x24:T24,_x25:T25,_x26:T26,_x27:T27,_x28:T28,_x29:T29,_x30:T30,_x31:T31){}}
impl<T0,T1,T2,T3,T4,T5,T6,T7,T8,T9,T10,T11,T12,T13,T14,T15,T16,T17,T18,T19,T20,T21,T22,T23,T24,T25,T26,T27,T28,T29,T30,T31> Batch<T0,T1,T2,T3,T4,T5,T6,T7,T8,T9,T10,T11,T12,T13,T14,T15,T16,T17,T18,T19,T20,T21,T22,T23,T24,T25,T26,T27,T28,T29,T30,T31> for u8{}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Batch<u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8,u8>>::consume(7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8);7}
