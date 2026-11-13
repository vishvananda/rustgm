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
// core/ptr/mod.rs:821: the compiler replaces this genuine glue declaration.
#[lang="drop_glue"]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="sync"]pub unsafe auto trait Sync{}
#[unsafe(no_mangle)]pub static STOP:ControlFlow<bool>=ControlFlow::Continue(());
#[unsafe(no_mangle)]pub static GO:ControlFlow<bool>=ControlFlow::Break(false);
#[unsafe(no_mangle)]pub extern "C" fn get_stop()->u8{match &STOP{ControlFlow::Continue(())=>2,ControlFlow::Break(b)=>*b as u8}}
#[unsafe(no_mangle)]pub extern "C" fn get_go()->u8{match &GO{ControlFlow::Continue(())=>2,ControlFlow::Break(b)=>*b as u8}}
