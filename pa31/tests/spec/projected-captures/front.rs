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
pub trait It{type Item;fn next(&mut self)->Option<Self::Item>;
fn fold<B,F:FnMut(B,Self::Item)->B>(mut self,init:B,mut f:F)->B where Self:Sized{let mut a=init;while let Option::Some(x)=self.next(){a=f(a,x);}a}}
pub struct One{pub v:u8,pub left:u8}
impl It for One{type Item=u8;fn next(&mut self)->Option<u8>{match self.left{0=>Option::None,_=>{self.left=0;Option::Some(self.v)}}}}
fn map_fold<'a,T,B,Acc>(mut f:impl FnMut(T)->B+'a,mut g:impl FnMut(Acc,B)->Acc+'a)->impl FnMut(Acc,T)->Acc+'a{move |acc,elt| g(acc,f(elt))}
pub struct Map<I,F>{pub iter:I,pub f:F}
pub trait IntoIt{type IntoIter:It;fn into_it(self)->Self::IntoIter;}
impl<I:It> IntoIt for I{type IntoIter=I;fn into_it(self)->I{self}}
pub fn take(it:&mut One)->u8{match it.next(){Option::Some(v)=>v,Option::None=>0}}
pub struct Outer{pub inner:Option<One>}
impl It for Outer{type Item=One;fn next(&mut self)->Option<One>{let v=match self.inner{Option::Some(ref o)=>Option::Some(One{v:o.v,left:o.left}),Option::None=>Option::None};self.inner=Option::None;v}}
fn wrap<'a,T:IntoIt,Acc>(front:&'a mut Option<T::IntoIter>,fold:&'a mut impl FnMut(Acc,&mut T::IntoIter)->Acc)->impl FnMut(Acc,T)->Acc+'a{move |acc,x|{let mut mid=x.into_it();let r=fold(acc,&mut mid);*front=Option::Some(mid);r}}
pub fn run(value:u8)->u8{let mut front:Option<One>=Option::None;let mut inner=|_acc:u8,it:&mut One|take(it);let mut w=wrap::<One,u8>(&mut front,&mut inner);w(0,One{v:value,left:1})}
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{run(41)}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{run(value)}
fn main(){}
