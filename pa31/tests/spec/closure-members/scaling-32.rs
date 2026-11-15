#![feature(const_closures)]
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
pub const trait Compare{fn compare(&self,other:&Self)->u8;}
const impl Compare for u8{fn compare(&self,_other:&Self)->u8{*self}}
impl Compare for u16{fn compare(&self,_other:&Self)->u8{0}}
pub const fn make0<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make1<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make2<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make3<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make4<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make5<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make6<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make7<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make8<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make9<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make10<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make11<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make12<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make13<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make14<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make15<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make16<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make17<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make18<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make19<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make20<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make21<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make22<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make23<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make24<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make25<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make26<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make27<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make28<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make29<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make30<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make31<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
fn main(){}
