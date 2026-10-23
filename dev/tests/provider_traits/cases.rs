//! Exact independently declared source cutouts, including retained required gaps.
#![allow(
    dead_code,
    reason = "each focused owner uses only its declared subset of the frozen matrix"
)]
pub const ABSENT_ROLE_LITERAL: &str = r#"#![feature(no_core,lang_items,const_trait_impl)]#![no_core]#![allow(internal_features,dead_code,unused_variables)]#[lang="pointee_sized"]pub trait PointeeSized{}#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}#[lang="sized"]pub trait Sized:MetaSized{}fn one()->u8{7}
"#;
pub const ABSENT_ROLE_ONE_USE: &str = r#"#![feature(no_core,lang_items,const_trait_impl)]#![no_core]#![allow(internal_features,dead_code,unused_variables)]#[lang="pointee_sized"]pub trait PointeeSized{}#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}#[lang="sized"]pub trait Sized:MetaSized{}fn one(x:u8)->u8{x}
"#;
pub const CALLABLE_FUNCTION_ITEM: &str = r#"#![feature(unboxed_closures)]
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
fn apply<F:FnOnce(u8)->u8>(f:F,x:u8)->u8{f(x)}fn id(x:u8)->u8{x}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{apply(id,23u8)}
"#;
pub const CONDITIONAL_RUNTIME_NONCONST_TARGET: &str = r#"#![feature(unboxed_closures)]
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
const fn apply<F:[const]FnOnce(u8)->u8>(f:F,x:u8)->u8{f(x)}fn id(x:u8)->u8{x}fn ordinary()->u8{apply(id,29u8)}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{ordinary()}
"#;
pub const CONST_CALLABLE_ITEM: &str = r#"#![feature(unboxed_closures)]
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
const fn apply<F:[const]FnOnce(u8)->u8>(f:F,x:u8)->u8{f(x)}const fn id(x:u8)->u8{x}const N:u8=apply(id,29u8);#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{N}
"#;
pub const COPY_IMPL_UNPAID_CLONE: &str = r#"#![feature(no_core,lang_items,const_trait_impl)]#![no_core]#![allow(internal_features,dead_code,unused_variables)]#[lang="pointee_sized"]pub trait PointeeSized{}#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}#[lang="sized"]pub trait Sized:MetaSized{}#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}impl<T:PointeeSized>LegacyReceiver for &T{}impl<T:PointeeSized>LegacyReceiver for &mut T{}#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}#[lang="copy"]pub trait Copy:Clone{}impl Copy for u8{}
"#;
pub const GENERIC_COPY: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
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
fn twice<T:Copy>(x:T)->T{let first=x;x}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{twice(7u8)}
"#;
pub const INHERITED_CONDITIONAL_HEADER: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
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
pub const trait Parent<T:[const]Clone>{fn seven()->u8{7}}
"#;
pub const MISSING_IMPL_DOUBLE_USE: &str = r#"#![feature(no_core,lang_items,const_trait_impl)]#![no_core]#![allow(internal_features,dead_code,unused_variables)]#[lang="pointee_sized"]pub trait PointeeSized{}#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}#[lang="sized"]pub trait Sized:MetaSized{}#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}impl<T:PointeeSized>LegacyReceiver for &T{}impl<T:PointeeSized>LegacyReceiver for &mut T{}#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}#[lang="copy"]pub trait Copy:Clone{}fn twice(x:u8)->u8{let first=x;x}
"#;
pub const MISSING_IMPL_ONE_USE: &str = r#"#![feature(no_core,lang_items,const_trait_impl)]#![no_core]#![allow(internal_features,dead_code,unused_variables)]#[lang="pointee_sized"]pub trait PointeeSized{}#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}#[lang="sized"]pub trait Sized:MetaSized{}#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}impl<T:PointeeSized>LegacyReceiver for &T{}impl<T:PointeeSized>LegacyReceiver for &mut T{}#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}#[lang="copy"]pub trait Copy:Clone{}fn one(x:u8)->u8{x}
"#;
pub const NONCONST_TRAIT_CONST_IMPL: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
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
trait Ordinary{}const impl Ordinary for u8{}
"#;
pub const NONCONST_TRAIT_QUALIFIED: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
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
trait Ordinary{}fn invalid<T:[const]Ordinary>(){}
"#;
pub const SOURCE_CLONE_UFCS: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
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
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Clone>::clone(&11u8)}
"#;
pub const UNPAID_CONST_DEFINING_BODY: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
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
const fn unused<T:[const]Clone>()->u8{7}#[unsafe(no_mangle)]pub const fn answer()->u8{unused::<u8>()}
"#;
pub const UNUSED_CONDITIONAL_CONST_PAID: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
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
const impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
const fn unused<T:[const]Clone>()->u8{7}const N:u8=unused::<u8>();#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{N}
"#;
pub const UNUSED_CONDITIONAL_CONST_UNPAID: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
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
const fn unused<T:[const]Clone>()->u8{7}const N:u8=unused::<u8>();#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{N}
"#;
pub const UNUSED_CONDITIONAL_RUNTIME: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
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
const fn unused<T:[const]Clone>()->u8{7}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{unused::<u8>()}
"#;
