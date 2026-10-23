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
struct Noise0([u8;1]);
struct Noise1([u8;2]);
struct Noise2([u8;3]);
struct Noise3([u8;4]);
struct Noise4([u8;5]);
struct Noise5([u8;6]);
struct Noise6([u8;7]);
struct Noise7([u8;8]);
struct Noise8([u8;9]);
struct Noise9([u8;10]);
struct Noise10([u8;11]);
struct Noise11([u8;12]);
struct Noise12([u8;13]);
struct Noise13([u8;14]);
struct Noise14([u8;15]);
struct Noise15([u8;16]);
struct Noise16([u8;17]);
struct Noise17([u8;18]);
struct Noise18([u8;19]);
struct Noise19([u8;20]);
struct Noise20([u8;21]);
struct Noise21([u8;22]);
struct Noise22([u8;23]);
struct Noise23([u8;24]);
struct Noise24([u8;25]);
struct Noise25([u8;26]);
struct Noise26([u8;27]);
struct Noise27([u8;28]);
struct Noise28([u8;29]);
struct Noise29([u8;30]);
struct Noise30([u8;31]);
struct Noise31([u8;32]);
struct Noise32([u8;33]);
struct Noise33([u8;34]);
struct Noise34([u8;35]);
struct Noise35([u8;36]);
struct Noise36([u8;37]);
struct Noise37([u8;38]);
struct Noise38([u8;39]);
struct Noise39([u8;40]);
struct Noise40([u8;41]);
struct Noise41([u8;42]);
struct Noise42([u8;43]);
struct Noise43([u8;44]);
struct Noise44([u8;45]);
struct Noise45([u8;46]);
struct Noise46([u8;47]);
struct Noise47([u8;48]);
struct Noise48([u8;49]);
struct Noise49([u8;50]);
struct Noise50([u8;51]);
struct Noise51([u8;52]);
struct Noise52([u8;53]);
struct Noise53([u8;54]);
struct Noise54([u8;55]);
struct Noise55([u8;56]);
struct Noise56([u8;57]);
struct Noise57([u8;58]);
struct Noise58([u8;59]);
struct Noise59([u8;60]);
struct Noise60([u8;61]);
struct Noise61([u8;62]);
struct Noise62([u8;63]);
struct Noise63([u8;64]);
const Q0:bool=needs_drop::<Root>();type Check0=[u8;Q0 as usize];
const Q1:bool=needs_drop::<Root>();type Check1=[u8;Q1 as usize];
const Q2:bool=needs_drop::<Root>();type Check2=[u8;Q2 as usize];
const Q3:bool=needs_drop::<Root>();type Check3=[u8;Q3 as usize];
const Q4:bool=needs_drop::<Root>();type Check4=[u8;Q4 as usize];
const Q5:bool=needs_drop::<Root>();type Check5=[u8;Q5 as usize];
const Q6:bool=needs_drop::<Root>();type Check6=[u8;Q6 as usize];
const Q7:bool=needs_drop::<Root>();type Check7=[u8;Q7 as usize];
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{Q7 as u8}
