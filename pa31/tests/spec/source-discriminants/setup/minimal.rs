#![feature(no_core,lang_items,rustc_attrs,intrinsics,core_intrinsics,auto_traits,staged_api,const_trait_impl,const_destruct)]
#![no_core]
#![unstable(feature="discriminant_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[unstable(feature="const_destruct",issue="133214")]
#[rustc_const_unstable(feature="const_destruct",issue="133214")]
#[lang="destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct:PointeeSized{}
impl Clone for i8{fn clone(&self)->Self{*self}}impl Copy for i8{}
impl Clone for isize{fn clone(&self)->Self{*self}}impl Copy for isize{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for u16{fn clone(&self)->Self{*self}}impl Copy for u16{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
// Authored no_core protocol: fewer written bounds are intentional and separately observed.
#[lang="discriminant_kind"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait DiscriminantKind { #[lang="discriminant_type"]type Discriminant:Copy; }
#[unstable(feature="core_intrinsics",issue="none")]
#[rustc_intrinsic_const_stable_indirect]#[rustc_nounwind]#[rustc_intrinsic]
pub const fn discriminant_value<T>(v:&T)-><T as DiscriminantKind>::Discriminant;
#[repr(i8)]pub enum Signed{Low=-1,Zero=0,High=1}
pub const HIGH:i8=discriminant_value(&Signed::High);
#[unsafe(no_mangle)]pub extern "C" fn answer()->i8{discriminant_value(&Signed::Low)}
