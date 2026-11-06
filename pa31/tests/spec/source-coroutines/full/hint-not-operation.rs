#![feature(no_core,lang_items,rustc_attrs)]#![no_core]#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]trait PointeeSized{}#[lang="meta_sized"]trait MetaSized:PointeeSized{}#[lang="sized"]trait Sized:MetaSized{}
#[rustc_diagnostic_item="PartialEq"]trait EqHint{}#[rustc_diagnostic_item="PartialOrd"]trait OrdHint{}fn compare<T:OrdHint>(a:T,b:T)->bool{a<b}