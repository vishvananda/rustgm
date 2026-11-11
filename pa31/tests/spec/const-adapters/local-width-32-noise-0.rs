// Authored reducer following pinned core/contracts.rs and callable language traits.
// Authored protocol reducer; interfaces follow pinned core/ops/try_trait.rs and control_flow.rs.
#![feature(unboxed_closures,auto_traits,const_closures,stmt_expr_attributes)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![feature(const_destruct)]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}

#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
const fn consume<F:[const]FnOnce(u8)->u8>(f:F)->u8{f(23)}
const fn local<T>(x:u8)->u8{
let call0=const move|v|x;consume(call0);
let call1=const move|v|x;consume(call1);
let call2=const move|v|x;consume(call2);
let call3=const move|v|x;consume(call3);
let call4=const move|v|x;consume(call4);
let call5=const move|v|x;consume(call5);
let call6=const move|v|x;consume(call6);
let call7=const move|v|x;consume(call7);
let call8=const move|v|x;consume(call8);
let call9=const move|v|x;consume(call9);
let call10=const move|v|x;consume(call10);
let call11=const move|v|x;consume(call11);
let call12=const move|v|x;consume(call12);
let call13=const move|v|x;consume(call13);
let call14=const move|v|x;consume(call14);
let call15=const move|v|x;consume(call15);
let call16=const move|v|x;consume(call16);
let call17=const move|v|x;consume(call17);
let call18=const move|v|x;consume(call18);
let call19=const move|v|x;consume(call19);
let call20=const move|v|x;consume(call20);
let call21=const move|v|x;consume(call21);
let call22=const move|v|x;consume(call22);
let call23=const move|v|x;consume(call23);
let call24=const move|v|x;consume(call24);
let call25=const move|v|x;consume(call25);
let call26=const move|v|x;consume(call26);
let call27=const move|v|x;consume(call27);
let call28=const move|v|x;consume(call28);
let call29=const move|v|x;consume(call29);
let call30=const move|v|x;consume(call30);
let call31=const move|v|x;consume(call31);
23}
