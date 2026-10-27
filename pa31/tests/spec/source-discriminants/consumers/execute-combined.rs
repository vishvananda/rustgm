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
// Actual primitive Neg declaration/implementation needed by the signed enum initializer.
#[lang="neg"]pub trait Neg{type Output;fn neg(self)->Self::Output;}
impl Neg for i8{type Output=i8;fn neg(self)->i8{-self}}
#[repr(i8)]pub enum Signed{Low=-1,Zero=0,High=1}
#[repr(u16)]pub enum Tagged{Empty=27,Value(u8)=300}
pub enum Maybe<'a>{Empty,Value(&'a u8)}
pub struct Data{value:u8}
pub const HIGH:i8=discriminant_value(&Signed::High);
pub const TAG:u16=discriminant_value(&Tagged::Value(7));
pub const ZERO:u8=discriminant_value(&Data{value:19});
pub const NONE:isize=discriminant_value(&Maybe::<'static>::Empty);
#[unsafe(no_mangle)]pub extern "C" fn signed()->i8{discriminant_value(&Signed::Low)}
#[unsafe(no_mangle)]pub extern "C" fn payload()->u16{discriminant_value(&Tagged::Value(7))}
#[unsafe(no_mangle)]pub extern "C" fn none()->isize{discriminant_value(&Maybe::<'static>::Empty)}
#[unsafe(no_mangle)]pub extern "C" fn some()->isize{let value=7u8;discriminant_value(&Maybe::Value(&value))}
#[unsafe(no_mangle)]pub extern "C" fn effects()->u8{let mut once=0u8;let _=discriminant_value(&{once=7u8;19u8});once}
trait Sink<T>{}impl Sink<i8> for u8{}
fn unresolved<T>(x:&T)-><T as DiscriminantKind>::Discriminant{discriminant_value(x)}

// Execution companion: the full independently observed provider is unchanged above.
unsafe extern "C" { fn observe(a:i8,b:u16,c:isize,d:isize,e:u8); }
#[unsafe(no_mangle)]pub extern "C" fn run(){unsafe{observe(signed(),payload(),none(),some(),effects());}}
fn main(){run();}
