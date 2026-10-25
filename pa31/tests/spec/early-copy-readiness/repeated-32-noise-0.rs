#![feature(unboxed_closures)]
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

impl Clone for u8{fn clone(&self)->Self{*self}}
impl Copy for u8{}
const fn read(x:&u8)->u8{*x}
type Read0=[u8;read(&7u8) as usize];
type Read1=[u8;read(&7u8) as usize];
type Read2=[u8;read(&7u8) as usize];
type Read3=[u8;read(&7u8) as usize];
type Read4=[u8;read(&7u8) as usize];
type Read5=[u8;read(&7u8) as usize];
type Read6=[u8;read(&7u8) as usize];
type Read7=[u8;read(&7u8) as usize];
type Read8=[u8;read(&7u8) as usize];
type Read9=[u8;read(&7u8) as usize];
type Read10=[u8;read(&7u8) as usize];
type Read11=[u8;read(&7u8) as usize];
type Read12=[u8;read(&7u8) as usize];
type Read13=[u8;read(&7u8) as usize];
type Read14=[u8;read(&7u8) as usize];
type Read15=[u8;read(&7u8) as usize];
type Read16=[u8;read(&7u8) as usize];
type Read17=[u8;read(&7u8) as usize];
type Read18=[u8;read(&7u8) as usize];
type Read19=[u8;read(&7u8) as usize];
type Read20=[u8;read(&7u8) as usize];
type Read21=[u8;read(&7u8) as usize];
type Read22=[u8;read(&7u8) as usize];
type Read23=[u8;read(&7u8) as usize];
type Read24=[u8;read(&7u8) as usize];
type Read25=[u8;read(&7u8) as usize];
type Read26=[u8;read(&7u8) as usize];
type Read27=[u8;read(&7u8) as usize];
type Read28=[u8;read(&7u8) as usize];
type Read29=[u8;read(&7u8) as usize];
type Read30=[u8;read(&7u8) as usize];
type Read31=[u8;read(&7u8) as usize];
type Early=Read0;
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{let _:Early=[0u8;7];7}
#[unsafe(no_mangle)]pub extern "C" fn runtime()->u8{read(&7u8)}
