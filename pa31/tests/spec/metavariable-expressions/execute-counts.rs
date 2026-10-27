#![feature(no_core,lang_items,macro_metavar_expr)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
macro_rules! flat{($($x:ident),*)=>{${count($x)}}}
macro_rules! nested{($( [$($x:ident),*] ),*)=>{${count($x)}}}
macro_rules! outer{($( [$($x:ident),*] ),*)=>{${count($x,1)}}}
macro_rules! selected{($($x:ident),*)=>{($( ${ignore($x)} (7usize,11usize,33usize).${index()}, )*)}}
#[unsafe(no_mangle)]pub extern "C" fn observed_flat()->usize{flat!(a,b,c)}
#[unsafe(no_mangle)]pub extern "C" fn observed_nested()->usize{nested!([a,b],[c])}
#[unsafe(no_mangle)]pub extern "C" fn observed_outer()->usize{outer!([a,b],[c])}
#[unsafe(no_mangle)]pub extern "C" fn observed_selected()->usize{selected!(a,b,c).2}
