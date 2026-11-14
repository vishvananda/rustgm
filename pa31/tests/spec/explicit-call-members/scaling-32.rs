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
pub struct Relay<F>{pub f:F}
const impl<F:[const]FnMut(u8)->u8>FnOnce<(u8,)> for &mut Relay<F>{type Output=u8;extern "rust-call" fn call_once(mut self,args:(u8,))->u8{self.call_mut(args)}}
const impl<F:[const]FnMut(u8)->u8>FnMut<(u8,)> for &mut Relay<F>{extern "rust-call" fn call_mut(&mut self,(value,):(u8,))->u8{(self.f)(value)}}
pub const fn invoke_0<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_1<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_2<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_3<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_4<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_5<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_6<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_7<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_8<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_9<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_10<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_11<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_12<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_13<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_14<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_15<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_16<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_17<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_18<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_19<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_20<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_21<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_22<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_23<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_24<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_25<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_26<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_27<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_28<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_29<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_30<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_31<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
fn main(){}
