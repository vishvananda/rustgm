#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl,auto_traits)]
#![no_core]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a mut U> for &'b mut T{}
impl<T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<*mut U> for *mut T{}
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

fn require<T:PointeeSized+Unsize<U>,U:PointeeSized>(){}
trait Flag{type V0;type V1;type V2;type V3;type V4;type V5;type V6;type V7;type V8;type V9;type V10;type V11;type V12;type V13;type V14;type V15;type V16;type V17;type V18;type V19;type V20;type V21;type V22;type V23;type V24;type V25;type V26;type V27;type V28;type V29;type V30;type V31;}struct A;impl Flag for A{type V0=u8;type V1=u8;type V2=u8;type V3=u8;type V4=u8;type V5=u8;type V6=u8;type V7=u8;type V8=u8;type V9=u8;type V10=u8;type V11=u8;type V12=u8;type V13=u8;type V14=u8;type V15=u8;type V16=u8;type V17=u8;type V18=u8;type V19=u8;type V20=u8;type V21=u8;type V22=u8;type V23=u8;type V24=u8;type V25=u8;type V26=u8;type V27=u8;type V28=u8;type V29=u8;type V30=u8;type V31=u8;}
type Target=dyn Flag<V0=u8,V1=u8,V2=u8,V3=u8,V4=u8,V5=u8,V6=u8,V7=u8,V8=u8,V9=u8,V10=u8,V11=u8,V12=u8,V13=u8,V14=u8,V15=u8,V16=u8,V17=u8,V18=u8,V19=u8,V20=u8,V21=u8,V22=u8,V23=u8,V24=u8,V25=u8,V26=u8,V27=u8,V28=u8,V29=u8,V30=u8,V31=u8>;
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{require::<A,Target>();7}
