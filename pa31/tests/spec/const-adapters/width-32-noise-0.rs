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
const fn one0<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one1<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one2<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one3<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one4<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one5<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one6<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one7<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one8<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one9<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one10<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one11<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one12<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one13<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one14<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one15<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one16<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one17<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one18<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one19<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one20<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one21<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one22<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one23<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one24<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one25<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one26<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one27<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one28<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one29<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one30<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one31<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
