// Authored source protocol; roles follow pinned core iter/traits and option.
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
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
#[lang="Option"]pub enum Maybe<T>{#[lang="Some"]Present(T),#[lang="None"]Absent}
#[lang="iterator"]pub trait Walk{type Item;fn advance(&mut self)->Maybe<Self::Item>;}
pub trait Start{type Item;type IntoIter:Walk<Item=Self::Item>;#[lang="into_iter"]fn begin(self)->Self::IntoIter;}
pub struct Values{pending:bool,value:u8}
impl Walk for Values{type Item=u8;fn advance(&mut self)->Maybe<u8>{if self.pending{self.pending=false;Maybe::Present(self.value)}else{Maybe::Absent}}}
impl Start for Values{type Item=u8;type IntoIter=Self;fn begin(self)->Self{self}}
#[unsafe(no_mangle)]pub extern "C" fn iteration_single()->u8{let mut last=0u8;for value in (Values{pending:true,value:7}){last=value;}last}
