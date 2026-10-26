#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="not"]pub trait OperatorNot{type Output;fn not(self)->Self::Output;}
pub struct Flag;impl OperatorNot for Flag{type Output=u8;fn not(self)->u8{7}}pub fn probe(x:Flag)->u8{if !x{1}else{0}}
