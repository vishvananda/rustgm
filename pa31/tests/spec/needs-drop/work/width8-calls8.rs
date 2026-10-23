#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
pub mod intrinsics{use crate::Sized;#[rustc_intrinsic]pub const fn needs_drop<T:?Sized>()->bool;}
pub const fn needs_drop<T: ?Sized>() -> bool {
    const { intrinsics::needs_drop::<T>() }
}

struct Leaf;impl Drop for Leaf{fn drop(&mut self){}}
struct Root(Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf);
const Q0:bool=needs_drop::<Root>();type Check0=[u8;Q0 as usize];
const Q1:bool=needs_drop::<Root>();type Check1=[u8;Q1 as usize];
const Q2:bool=needs_drop::<Root>();type Check2=[u8;Q2 as usize];
const Q3:bool=needs_drop::<Root>();type Check3=[u8;Q3 as usize];
const Q4:bool=needs_drop::<Root>();type Check4=[u8;Q4 as usize];
const Q5:bool=needs_drop::<Root>();type Check5=[u8;Q5 as usize];
const Q6:bool=needs_drop::<Root>();type Check6=[u8;Q6 as usize];
const Q7:bool=needs_drop::<Root>();type Check7=[u8;Q7 as usize];
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{Q7 as u8}
