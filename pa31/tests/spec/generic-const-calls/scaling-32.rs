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
pub const fn apply0<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply1<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply2<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply3<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply4<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply5<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply6<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply7<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply8<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply9<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply10<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply11<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply12<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply13<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply14<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply15<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply16<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply17<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply18<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply19<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply20<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply21<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply22<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply23<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply24<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply25<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply26<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply27<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply28<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply29<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply30<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply31<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
fn main(){}
