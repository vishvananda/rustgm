#![feature(const_destruct)]
#![feature(auto_traits)]
#![feature(unboxed_closures,const_trait_impl)]
#![feature(no_core,lang_items,rustc_attrs,fundamental)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang = "pointee_sized"]
#[diagnostic::on_unimplemented(
    message = "values of type `{Self}` may or may not have a size",
    label = "may or may not have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_coinductive]
pub trait PointeeSized {
    // Empty
}
#[lang = "meta_sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known",
    label = "doesn't have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
// `MetaSized` being coinductive, despite having supertraits, is okay for the same reasons as
// `Sized` above.
#[rustc_coinductive]
pub trait MetaSized: PointeeSized {
    // Empty
}
#[lang = "sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known at compilation time",
    label = "doesn't have a size known at compile-time"
)]
#[fundamental] // for Default, for example, which requires that `[T]: !Default` be evaluatable
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// `Sized` being coinductive, despite having supertraits, is okay as there are no user-written impls,
// and we know that the supertraits are always implemented if the subtrait is just by looking at
// the builtin impls.
#[rustc_coinductive]
pub trait Sized: MetaSized {
    // Empty.
}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}
pub fn identity<T>(x: T) -> T { x }

impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}

impl<T:PointeeSized> Clone for &T{fn clone(&self)->Self{*self}}
impl<T:PointeeSized> Copy for &T{}

#[lang="freeze"]pub unsafe auto trait Freeze{}

#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}

#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
#[lang="deref"]pub const trait Deref:PointeeSized{#[lang="deref_target"]type Target:PointeeSized;fn deref(&self)->&Self::Target;}
#[lang="deref_mut"]pub const trait DerefMut:Deref+PointeeSized{fn deref_mut(&mut self)->&mut Self::Target;}
struct Guard<'a,T>(&'a T);impl<'a,T>Deref for Guard<'a,T>{type Target=T;fn deref(&self)->&T{self.0}}
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
fn choose<U>(_:&U,value:U)->U{value}
fn exercise<T:Clone>(x:Guard<'_,T>)->T{let value:T=x.clone();let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);let value:T=choose(&x,value);value}
