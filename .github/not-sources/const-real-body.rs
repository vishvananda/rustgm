#![feature(auto_traits)]
#![feature(const_trait_impl)]
#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="sync"]pub unsafe auto trait Sync{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
#[lang="destruct"]pub const trait Destruct:PointeeSized{}
#[lang="not"]pub const trait OperatorNot{type Output;fn not(self)->Self::Output;}
const impl OperatorNot for bool{type Output=bool;fn not(self)->bool{!self}}
#[unsafe(no_mangle)]pub static ANSWER:bool=!false;

