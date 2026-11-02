#![feature(staged_api,contracts_internals)]
#![unstable(feature="contract_fixture",issue="none")]
// Authored reducer following pinned core/contracts.rs and callable language traits.
// Authored protocol reducer; interfaces follow pinned core/ops/try_trait.rs and control_flow.rs.
#![feature(unboxed_closures,auto_traits)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]#[rustc_const_unstable(feature="const_trait_impl",issue="none")]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[lang="fn_once"]#[rustc_const_unstable(feature="const_trait_impl",issue="none")]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[lang="fn_mut"]#[rustc_const_unstable(feature="const_trait_impl",issue="none")]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[lang="fn"]#[rustc_const_unstable(feature="const_trait_impl",issue="none")]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}

#[lang="freeze"]pub unsafe auto trait Freeze{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
#[unstable(feature="contracts_internals",issue="128044")]
#[rustc_const_unstable(feature="contracts",issue="128044")]
#[lang="contract_build_check_ensures"]pub const fn make_predicate<Ret,C>(cond:C)->C where C:Fn(&Ret)->bool+Copy+'static{cond}
fn predicate(_: &u8)->bool{true}
const VALUE:fn(&u8)->bool=make_predicate::<u8,_>(predicate);
#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{if VALUE(&7){7}else{3}}
