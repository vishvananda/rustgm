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
#[lang="deref"]pub trait Deref:PointeeSized{#[lang="deref_target"]type Target:PointeeSized;fn deref(&self)->&Self::Target;}
#[lang="deref_mut"]pub trait DerefMut:Deref+PointeeSized{fn deref_mut(&mut self)->&mut Self::Target;}

unsafe extern "C"{fn record(value:u8);}
pub struct Leaf{pub value:u8}
pub struct Inner{pub leaf:Leaf}
pub struct Wrap{pub inner:Inner}
impl Deref for Wrap{type Target=Inner;fn deref(&self)->&Inner{unsafe{record(1)};&self.inner}}
impl DerefMut for Wrap{fn deref_mut(&mut self)->&mut Inner{unsafe{record(2)};&mut self.inner}}
impl Leaf{pub fn set(&mut self){self.value=23}}
fn make()->Wrap{unsafe{record(3)};Wrap{inner:Inner{leaf:Leaf{value:13}}}}
pub struct Outer{pub inner:Wrap}
impl Deref for Outer{type Target=Wrap;fn deref(&self)->&Wrap{unsafe{record(4)};&self.inner}}
pub struct Shadow{pub leaf:Leaf,pub inner:Inner}
impl Deref for Shadow{type Target=Inner;fn deref(&self)->&Inner{unsafe{record(5)};&self.inner}}
#[unsafe(no_mangle)]pub extern "C" fn field_shared()->u8{let x=Wrap{inner:Inner{leaf:Leaf{value:13}}};x.leaf.value}
#[unsafe(no_mangle)]pub extern "C" fn field_late_mut()->u8{let mut x=Wrap{inner:Inner{leaf:Leaf{value:13}}};x.leaf.set();x.inner.leaf.value}
#[unsafe(no_mangle)]pub extern "C" fn field_temporary()->u8{make().leaf.value}
#[unsafe(no_mangle)]pub extern "C" fn field_nested()->u8{let x=Outer{inner:Wrap{inner:Inner{leaf:Leaf{value:13}}}};x.leaf.value}
#[unsafe(no_mangle)]pub extern "C" fn field_shadow()->u8{let x=Shadow{leaf:Leaf{value:41},inner:Inner{leaf:Leaf{value:13}}};x.leaf.value}
