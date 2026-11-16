#![feature(const_destruct,const_precise_live_drops)]
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
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
pub trait Pattern:Sized{type Searcher;fn into_searcher(self,start:usize)->Self::Searcher;}
pub struct ByteSearcher{position:usize,needle:u8}
impl Pattern for u8{type Searcher=ByteSearcher;fn into_searcher(self,start:usize)->ByteSearcher{ByteSearcher{position:start,needle:self}}}
pub struct SplitInternal<P:Pattern>{start:usize,end:usize,matcher:P::Searcher,allow_trailing_empty:bool,finished:bool}
pub struct Split<P:Pattern>(pub SplitInternal<P>);
pub struct SplitTerminator<P:Pattern>(pub SplitInternal<P>);
pub fn split<P:Pattern>(pat:P)->Split<P>{Split(SplitInternal{start:0,end:1,matcher:pat.into_searcher(0),allow_trailing_empty:true,finished:false})}
pub struct Plain<T>{start:usize,value:T,flag:bool}
pub fn update<T>(value:T)->T{let base=Plain{start:0,value,flag:true};let moved=Plain{flag:false,..base};base.value}
pub fn run(value:u8)->u8{value}
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{run(41)}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{run(value)}
fn main(){}
