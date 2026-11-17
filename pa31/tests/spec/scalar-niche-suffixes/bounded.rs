#![feature(no_core,lang_items,intrinsics,rustc_attrs,pattern_types,auto_traits,decl_macro,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables,incomplete_features)]
// Authored reducer of core's io/error/repr_bitpacked.rs static_asserts and slice/cmp.rs
// `as_underlying` (pinned core 88d9e12a): NonNull, PhantomData, Result and ControlFlow
// shapes follow ptr/non_null.rs, marker.rs, result.rs and ops/control_flow.rs.
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="destruct"]pub const trait Destruct:PointeeSized{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
#[lang="phantom_data"]pub struct PhantomData<T:PointeeSized>;
#[rustc_intrinsic]pub const fn size_of<T>()->usize;
#[rustc_intrinsic]pub const unsafe fn transmute<Src,Dst>(src:Src)->Dst;
#[rustc_builtin_macro(pattern_type)]pub macro pattern_type($($arg:tt)*){}
#[repr(transparent)]#[rustc_nonnull_optimization_guaranteed]
pub struct NonNull<T:PointeeSized>{pointer:pattern_type!(*const T is !null)}
pub enum Result<T,E>{Ok(T),Err(E)}
pub use Result::{Ok,Err};
#[lang="Option"]pub enum Option<T>{#[lang="None"]None,#[lang="Some"]Some(T)}
pub enum ControlFlow<B,C=()>{Continue(C),Break(B)}
pub struct Custom;
pub enum ErrorData<C>{Os(i32),Custom(C)}
#[repr(transparent)]pub struct Repr(NonNull<()>,PhantomData<ErrorData<Custom>>);
#[repr(transparent)]pub struct Bounded(pattern_type!(u32 is 0u32..=0x10ffffu32));
const _: [u8; size_of::<Option<Bounded>>()]=[0;4];
const _: [u8; match unsafe {transmute::<Option<Bounded>,u32>(Option::None)} {0xffffffffu32=>1,_=>0}]=[0;1];
const _: [u8; match unsafe {transmute::<u32,Option<Bounded>>(0u32)} {Option::Some(_)=>1,Option::None=>0}]=[0;1];
#[unsafe(no_mangle)]pub extern "C" fn pass(value:Option<Bounded>)->Option<Bounded>{value}
#[unsafe(no_mangle)]pub extern "C" fn first()->u8{
 match unsafe {transmute::<Option<Bounded>,u32>(pass(Option::None))} {0xffffffffu32=>41,_=>0}
}
#[unsafe(no_mangle)]pub extern "C" fn second()->u8{
 let value=pass(Option::Some(Bounded(unsafe {transmute(59u32)})));
 match value {Option::Some(value)=>match unsafe {transmute::<Bounded,u32>(value)} {59u32=>59,_=>0},Option::None=>0}
}
