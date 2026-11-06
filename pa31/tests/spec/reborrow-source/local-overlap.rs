#![feature(no_core,lang_items,rustc_attrs,staged_api,auto_traits,const_trait_impl,reborrow)]
#![no_core]
#![unstable(feature="reborrow_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for &T{}
/// Allows value to be reborrowed as exclusive, creating a copy of the value
/// that disables the source for reads and writes for the lifetime of the copy.
#[lang = "reborrow"]
#[unstable(feature = "reborrow", issue = "145612")]
pub trait Reborrow {
    /* compiler built-in */
}

/// Allows reborrowable value to be reborrowed as shared, creating a copy
/// that disables the source for writes for the lifetime of the copy.
#[lang = "coerce_shared"]
#[unstable(feature = "reborrow", issue = "145612")]
pub trait CoerceShared<Target: Copy>: Reborrow {
    /* compiler built-in */
}

struct Exclusive<'a>{value:&'a mut u8}
impl<'a>Reborrow for Exclusive<'a>{}
struct Shared<'a>{value:&'a u8}
static mut CLONES:u8=0;
impl Clone for Shared<'_>{fn clone(&self)->Self{unsafe{CLONES=99;}Self{value:self.value}}}
impl Copy for Shared<'_>{}
impl<'a>CoerceShared<Shared<'a>>for Exclusive<'a>{}
fn write(value:Exclusive<'_>,next:u8){*value.value=next;}
fn read(value:Shared<'_>)->u8{*value.value}
fn keep<'a>(value:Exclusive<'a>)->&'a mut u8{value.value}
pub fn bad(value:&mut u8){let value=Exclusive{value};let a=keep(value);let b=keep(value);std::hint::black_box((a,b));}
