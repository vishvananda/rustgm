// Authored protocol reducer; interfaces follow pinned core/ops/try_trait.rs and control_flow.rs.
#![feature(try_blocks)]
#![feature(unboxed_closures)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
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

pub enum ControlFlow<B,C>{#[lang="Continue"]Continue(C),#[lang="Break"]Break(B)}
pub trait FromResidual<R=<Self as Try>::Residual>{#[lang="from_residual"]fn from_residual(value:R)->Self;}
#[lang="Try"]pub trait Try:FromResidual{
    type Output;
    type Residual:Residual<Self::Output>;
    #[lang="from_output"]fn from_output(value:Self::Output)->Self;
    #[lang="branch"]fn branch(self)->ControlFlow<Self::Residual,Self::Output>;
}
pub trait Residual<O>:Sized{type TryType:Try<Output=O,Residual=Self>;}
#[lang="into_try_type"]pub fn residual_into_try_type<R:Residual<O>,O>(value:R)-><R as Residual<O>>::TryType{
    <<R as Residual<O>>::TryType as FromResidual<R>>::from_residual(value)
}
pub enum Outcome<T>{Value(T),Error(u8)}
pub enum Short{Error(u8)}
impl<T> Residual<T> for Short{type TryType=Outcome<T>;}
impl<T> FromResidual<Short> for Outcome<T>{fn from_residual(_value:Short)->Self{Outcome::Error(23)}}
impl<T> Try for Outcome<T>{
    type Output=T;type Residual=Short;
    fn from_output(value:T)->Self{Outcome::Value(value)}
    fn branch(self)->ControlFlow<Short,T>{match self{Outcome::Value(value)=>ControlFlow::Continue(value),Outcome::Error(error)=>ControlFlow::Break(Short::Error(error))}}
}
pub struct Weird(u8);
pub enum WeirdResidual{Error(u8)}
impl Residual<u8> for WeirdResidual{type TryType=Weird;}
impl FromResidual<WeirdResidual> for Weird{fn from_residual(_value:WeirdResidual)->Self{Weird(47)}}
impl Try for Weird{
    type Output=u8;type Residual=WeirdResidual;
    fn from_output(_value:u8)->Self{Weird(43)}
    fn branch(self)->ControlFlow<WeirdResidual,u8>{ControlFlow::Continue(self.0)}
}
fn code(value:Outcome<u8>)->u8{match value{Outcome::Value(value)=>value,Outcome::Error(error)=>error}}

#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
unsafe extern "C"{fn record(value:u8);}
pub struct Tracked(u8);
impl Drop for Tracked{fn drop(&mut self){unsafe{record(self.0)}}}
#[unsafe(no_mangle)]pub extern "C" fn try_drop(){
 let value:Outcome<u8>=try{
  let _before=Tracked(1);
  loop{let _inside=Tracked(2);let _:u8=Outcome::<u8>::Error(9)?;break;}
  let _after=Tracked(3);7
 };let _value=value;
}
