#![feature(const_destruct)]
#![feature(auto_traits)]
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

#[lang = "freeze"]
pub unsafe auto trait Freeze {}
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
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang = "drop_glue"]
pub(crate) const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
}

fn make(value:u8)->impl Fn()->u8{move||{let _r0:Outcome<u8>=try{value};let _r1:Outcome<u8>=try{value};let _r2:Outcome<u8>=try{value};let _r3:Outcome<u8>=try{value};let _r4:Outcome<u8>=try{value};let _r5:Outcome<u8>=try{value};let _r6:Outcome<u8>=try{value};let _r7:Outcome<u8>=try{value};value}}
struct Noise0;
struct Noise1;
struct Noise2;
struct Noise3;
struct Noise4;
struct Noise5;
struct Noise6;
struct Noise7;
struct Noise8;
struct Noise9;
struct Noise10;
struct Noise11;
struct Noise12;
struct Noise13;
struct Noise14;
struct Noise15;
struct Noise16;
struct Noise17;
struct Noise18;
struct Noise19;
struct Noise20;
struct Noise21;
struct Noise22;
struct Noise23;
struct Noise24;
struct Noise25;
struct Noise26;
struct Noise27;
struct Noise28;
struct Noise29;
struct Noise30;
struct Noise31;
struct Noise32;
struct Noise33;
struct Noise34;
struct Noise35;
struct Noise36;
struct Noise37;
struct Noise38;
struct Noise39;
struct Noise40;
struct Noise41;
struct Noise42;
struct Noise43;
struct Noise44;
struct Noise45;
struct Noise46;
struct Noise47;
struct Noise48;
struct Noise49;
struct Noise50;
struct Noise51;
struct Noise52;
struct Noise53;
struct Noise54;
struct Noise55;
struct Noise56;
struct Noise57;
struct Noise58;
struct Noise59;
struct Noise60;
struct Noise61;
struct Noise62;
struct Noise63;
struct Noise64;
struct Noise65;
struct Noise66;
struct Noise67;
struct Noise68;
struct Noise69;
struct Noise70;
struct Noise71;
struct Noise72;
struct Noise73;
struct Noise74;
struct Noise75;
struct Noise76;
struct Noise77;
struct Noise78;
struct Noise79;
struct Noise80;
struct Noise81;
struct Noise82;
struct Noise83;
struct Noise84;
struct Noise85;
struct Noise86;
struct Noise87;
struct Noise88;
struct Noise89;
struct Noise90;
struct Noise91;
struct Noise92;
struct Noise93;
struct Noise94;
struct Noise95;
struct Noise96;
struct Noise97;
struct Noise98;
struct Noise99;
struct Noise100;
struct Noise101;
struct Noise102;
struct Noise103;
struct Noise104;
struct Noise105;
struct Noise106;
struct Noise107;
struct Noise108;
struct Noise109;
struct Noise110;
struct Noise111;
struct Noise112;
struct Noise113;
struct Noise114;
struct Noise115;
struct Noise116;
struct Noise117;
struct Noise118;
struct Noise119;
struct Noise120;
struct Noise121;
struct Noise122;
struct Noise123;
struct Noise124;
struct Noise125;
struct Noise126;
struct Noise127;
