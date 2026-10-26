#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="not"]pub trait OperatorNot{type Output;fn not(self)->Self::Output;}
impl OperatorNot for bool{type Output=bool;fn not(self)->bool{!self}}
pub struct Flag(bool);impl OperatorNot for Flag{type Output=bool;fn not(self)->bool{!self.0}}pub fn probe(x:Flag)->bool{!x}
