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

pub const trait Family<'a>{type Item0;fn consume(x0:Self::Item0,x1:Self::Item0,x2:Self::Item0,x3:Self::Item0,x4:Self::Item0,x5:Self::Item0,x6:Self::Item0,x7:Self::Item0,x8:Self::Item0,x9:Self::Item0,x10:Self::Item0,x11:Self::Item0,x12:Self::Item0,x13:Self::Item0,x14:Self::Item0,x15:Self::Item0,x16:Self::Item0,x17:Self::Item0,x18:Self::Item0,x19:Self::Item0,x20:Self::Item0,x21:Self::Item0,x22:Self::Item0,x23:Self::Item0,x24:Self::Item0,x25:Self::Item0,x26:Self::Item0,x27:Self::Item0,x28:Self::Item0,x29:Self::Item0,x30:Self::Item0,x31:Self::Item0)where Self::Item0:[const]Destruct{let _moved0=x0;let _moved1=x1;let _moved2=x2;let _moved3=x3;let _moved4=x4;let _moved5=x5;let _moved6=x6;let _moved7=x7;let _moved8=x8;let _moved9=x9;let _moved10=x10;let _moved11=x11;let _moved12=x12;let _moved13=x13;let _moved14=x14;let _moved15=x15;let _moved16=x16;let _moved17=x17;let _moved18=x18;let _moved19=x19;let _moved20=x20;let _moved21=x21;let _moved22=x22;let _moved23=x23;let _moved24=x24;let _moved25=x25;let _moved26=x26;let _moved27=x27;let _moved28=x28;let _moved29=x29;let _moved30=x30;let _moved31=x31;}}
impl<'a>Family<'a>for u8{type Item0=u8;}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Family<'_>>::consume(7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8);7}
