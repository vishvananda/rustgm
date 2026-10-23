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
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{require::<[u8;3],[u8]>();7}

trait Paid{}struct Has;impl Paid for Has{}struct Missing;struct Checked<T:Paid>{v:T}struct Plain;struct Borrowed<'a,'b>where 'a:'b{a:&'a u8,b:&'b u8}trait Project{type Out:PointeeSized;}struct Projected<'a,'b>{a:&'a u8,b:&'b u8}impl<'a,'b>Project for Projected<'a,'b>where 'a:'b{type Out=Plain;}
fn check<'a,'b>(){require::<Borrowed<'a,'b>,u8>();}
