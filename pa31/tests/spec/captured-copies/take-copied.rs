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
const impl<A:Tuple,F:?Sized>FnMut<A> for &mut F where F:[const]FnMut<A>{extern "rust-call" fn call_mut(&mut self,args:A)->F::Output{(*self).call_mut(args)}}
const impl<A:Tuple,F:?Sized>FnOnce<A> for &mut F where F:[const]FnMut<A>{type Output=F::Output;extern "rust-call" fn call_once(self,args:A)->F::Output{(*self).call_mut(args)}}
pub enum Option<T>{None,Some(T)}
pub enum ControlFlow<B,C>{Continue(C),Break(B)}
pub trait Try{type Output;fn from_output(o:Self::Output)->Self;fn into_flow(self)->ControlFlow<Self,Self::Output> where Self:Sized;}
impl<B,C> Try for ControlFlow<B,C>{type Output=C;fn from_output(c:C)->Self{ControlFlow::Continue(c)}fn into_flow(self)->ControlFlow<Self,C>{match self{ControlFlow::Continue(c)=>ControlFlow::Continue(c),ControlFlow::Break(b)=>ControlFlow::Break(ControlFlow::Break(b))}}}
impl<B,C> ControlFlow<B,C>{pub fn into_try<R:Try<Output=C>>(self)->R where B:Into<R>{match self{ControlFlow::Continue(c)=>R::from_output(c),ControlFlow::Break(b)=>b.into()}}}
pub trait Into<T>{fn into(self)->T;}
impl<T> Into<T> for T{fn into(self)->T{self}}
pub trait It{type Item;fn next(&mut self)->Option<Self::Item>;
fn try_fold<B,F,R>(&mut self,init:B,mut f:F)->R where Self:Sized,F:FnMut(B,Self::Item)->R,R:Try<Output=B>{let mut a=init;while let Option::Some(x)=self.next(){a=match f(a,x).into_flow(){ControlFlow::Continue(c)=>c,ControlFlow::Break(b)=>return b};}R::from_output(a)}}
pub struct Refs<'a>{pub v:&'a u8,pub left:u8}
impl<'a> It for Refs<'a>{type Item=&'a u8;fn next(&mut self)->Option<&'a u8>{match self.left{0=>Option::None,_=>{self.left=0;Option::Some(self.v)}}}}
pub struct Take<I>{pub iter:I,pub n:u8}
impl<I:It> It for Take<I>{type Item=I::Item;fn next(&mut self)->Option<I::Item>{match self.n{0=>Option::None,_=>{self.n=0;self.iter.next()}}}
fn try_fold<Acc,Fold,R>(&mut self,init:Acc,fold:Fold)->R where Self:Sized,Fold:FnMut(Acc,Self::Item)->R,R:Try<Output=Acc>{
fn check<'a,T,Acc,R:Try<Output=Acc>>(n:&'a mut u8,mut fold:impl FnMut(Acc,T)->R+'a)->impl FnMut(Acc,T)->ControlFlow<R,Acc>+'a{move |acc,x|{let last=*n;*n=0;let r=fold(acc,x);match last{1=>ControlFlow::Break(r),_=>match r.into_flow(){ControlFlow::Continue(c)=>ControlFlow::Continue(c),ControlFlow::Break(b)=>ControlFlow::Break(b)}}}}
match self.n{0=>R::from_output(init),_=>{let n=&mut self.n;self.iter.try_fold(init,check(n,fold)).into_try()}}}}
pub struct Never<T>(pub T);
impl<T> Try for Never<T>{type Output=T;fn from_output(t:T)->Self{Never(t)}fn into_flow(self)->ControlFlow<Self,T>{ControlFlow::Continue(self.0)}}
fn wrap<A,B,T,F:FnMut(A,B)->T>(mut f:F)->impl FnMut(A,B)->Never<T>{move|a,b|Never(f(a,b))}
fn copy_fold<T:Copy,Acc>(mut f:impl FnMut(Acc,T)->Acc)->impl FnMut(Acc,&T)->Acc{move|acc,&elt|f(acc,elt)}
pub fn run(value:u8)->u8{let Never(r)=Take{iter:Refs{v:&value,left:1},n:2}.try_fold(0u8,wrap(copy_fold(|_a:u8,x:u8|x)));r}
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{run(41)}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{run(value)}
fn main(){}
